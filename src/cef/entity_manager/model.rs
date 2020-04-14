use super::{CEF_HEIGHT, CEF_WIDTH, TEXTURE_HEIGHT, TEXTURE_WIDTH};
use crate::helpers::*;
use classicube_sys::{
    Bitmap, Entity, Model, ModelTex, ModelVertex, Model_Init, Model_Register, OwnedGfxTexture,
    PackedCol, PackedCol_Make, Texture, TextureRec, MODEL_BOX_VERTICES,
};
use std::{ffi::CString, mem, pin::Pin};

const WHITE_TRANSPARENT: PackedCol = PackedCol_Make(255, 255, 255, 0);

pub struct CefModel {
    name: Pin<Box<CString>>,
    default_texture_name: Pin<Box<CString>>,
    model: Pin<Box<Model>>,
    vertices: Pin<Box<[ModelVertex; MODEL_BOX_VERTICES as usize]>>,
    default_model_tex: Pin<Box<ModelTex>>,

    default_texture: Option<OwnedGfxTexture>,
}

impl CefModel {
    pub fn register() -> Self {
        let name = "cef";
        let default_texture_name = format!("{}_texture", name);

        let model = Box::pin(unsafe { mem::zeroed() });
        let name = Box::pin(CString::new(name).unwrap());
        let default_texture_name = Box::pin(CString::new(default_texture_name).unwrap());
        let vertices = Box::pin(unsafe { mem::zeroed() });
        let default_model_tex = Box::pin(unsafe { mem::zeroed() });

        let mut this = Self {
            model,
            name,
            default_texture_name,
            vertices,
            default_model_tex,
            default_texture: None,
        };

        unsafe {
            this.register_gfx_texture();
            this.register_texture();
            this.register_model();
        }

        this
    }
}

impl CefModel {
    unsafe fn register_gfx_texture(&mut self) {
        // must be a vec or else we try to fit huge array onto stack and crash!
        let mut pixels: Vec<u8> = vec![255; 4 * TEXTURE_WIDTH * TEXTURE_HEIGHT];

        let mut bmp = Bitmap {
            Scan0: pixels.as_mut_ptr(),
            Width: TEXTURE_WIDTH as i32,
            Height: TEXTURE_HEIGHT as i32,
        };

        let default_texture = OwnedGfxTexture::create(&mut bmp, true, false);

        self.default_texture = Some(default_texture);
    }

    unsafe fn register_texture(&mut self) {
        let CefModel {
            default_model_tex,
            default_texture_name,
            ..
        } = self;

        default_model_tex.name = default_texture_name.as_ptr();
        default_model_tex.skinType = classicube_sys::SKIN_TYPE_SKIN_64x64 as _;
        default_model_tex.texID = self.default_texture.as_mut().unwrap().resource_id;

        // we don't need to register our texture!
        // Model_RegisterTexture(default_model_tex.as_mut().get_unchecked_mut());
    }

    extern "C" fn draw(entity: *mut Entity) {
        let entity = unsafe { &mut *entity };

        let resource_id = entity.TextureId;

        let mut tex = Texture {
            ID: resource_id,
            X: -8,
            Y: -9,
            // TODO have these be dynamic, they're block sizes
            // and use a pixels per block for resolution
            // ACTUALLY let's try using ModelScale on Entity and keep this 16:9 like the CEF resolution is
            Width: 16,
            Height: 9,
            uv: TextureRec {
                U1: 0.0,
                V1: 0.0,
                U2: CEF_WIDTH as f32 / TEXTURE_WIDTH as f32,
                V2: CEF_HEIGHT as f32 / TEXTURE_HEIGHT as f32,
            },
        };

        unsafe {
            Texture_RenderShaded(&mut tex, WHITE_TRANSPARENT);
        }
    }

    unsafe fn register_model(&mut self) {
        let CefModel {
            default_model_tex,
            model,
            vertices,
            name,
            ..
        } = self;

        model.name = name.as_ptr();
        model.vertices = vertices.as_mut_ptr();
        model.defaultTex = default_model_tex.as_mut().get_unchecked_mut();

        extern "C" fn make_parts() {}
        model.MakeParts = Some(make_parts);

        model.Draw = Some(Self::draw);

        extern "C" fn get_name_y(_entity: *mut Entity) -> f32 {
            0.0
        }
        model.GetNameY = Some(get_name_y);

        extern "C" fn get_eye_y(_entity: *mut Entity) -> f32 {
            0.0
        }
        model.GetEyeY = Some(get_eye_y);

        extern "C" fn get_collision_size(_entity: *mut Entity) {}
        model.GetCollisionSize = Some(get_collision_size);

        extern "C" fn get_picking_bounds(_entity: *mut Entity) {}
        model.GetPickingBounds = Some(get_picking_bounds);

        Model_Init(model.as_mut().get_unchecked_mut());

        model.bobbing = 0;

        println!("Model_Register {:#?}", model);
        Model_Register(model.as_mut().get_unchecked_mut());
    }
}
