use fingerprint_api_noise::*;

#[test]
fn test_canvas_noise_consistency() {
    let injector = CanvasNoiseInjector::new(12345, 0.1);
    let data = vec![255u8; 1000];

    // 同样of种子应该产生同样of噪声
    let noisy1 = injector.add_noise(&data);
    let noisy2 = injector.add_noise(&data);

    assert_eq!(noisy1, noisy2);
}

#[test]
fn test_different_seeds_different_noise() {
    let injector1 = CanvasNoiseInjector::new(12345, 0.1);
    let injector2 = CanvasNoiseInjector::new(67890, 0.1);
    let data = vec![255u8; 1000];

    let noisy1 = injector1.add_noise(&data);
    let noisy2 = injector2.add_noise(&data);

    assert_ne!(noisy1, noisy2);
}

#[test]
fn test_font_noise_variation() {
    let injector = FontNoiseInjector::new();

    let fonts1 = injector.get_fonts_with_noise(111);
    let fonts2 = injector.get_fonts_with_noise(222);

    // 不同种子应该产生不同offontlist
    assert_ne!(fonts1, fonts2);
}

#[test]
fn test_canvas_fingerprint_hash() {
    let injector = CanvasNoiseInjector::new(12345, 0.1);
    let data = vec![255u8; 1000];

    let hash1 = injector.fingerprint_hash(&data);
    let hash2 = injector.fingerprint_hash(&data);

    // 同样of种子应该产生同样ofhash
    assert_eq!(hash1, hash2);

    // hash应该是有效of SHA256 hash (64 个十六进制字符)
    assert_eq!(hash1.len(), 64);
}

#[test]
fn test_audio_noise_consistency() {
    let injector = AudioNoiseInjector::new(12345);
    let samples = vec![0.5f32; 100];

    let noisy1 = injector.add_audio_noise(&samples);
    let noisy2 = injector.add_audio_noise(&samples);

    assert_eq!(noisy1, noisy2);
}

#[test]
fn test_webgl_params_noise() {
    let injector = WebGLNoiseInjector::new();
    let params = webgl::WebGLParams {
        renderer: "ANGLE (Intel, Intel(R) UHD Graphics Direct3D11)".to_string(),
        vendor: "Google Inc. (Intel)".to_string(),
        aliased_line_width_range: Some([1.0, 1.0]),
        aliased_point_size_range: Some([1.0, 1024.0]),
        max_texture_size: Some(16384),
        max_viewport_dims: Some([16384, 16384]),
    };

    let noisy_params = injector.add_webgl_noise(&params);

    // 基本字段应该保持不变
    assert_eq!(noisy_params.renderer, params.renderer);
    assert_eq!(noisy_params.vendor, params.vendor);
}

#[test]
fn test_webgl_noise_reproducibility() {
    let injector1 = webgl::WebGLNoiseInjector::with_seed(12345);
    let injector2 = webgl::WebGLNoiseInjector::with_seed(12345);

    let params = webgl::WebGLParams {
        renderer: "Test Renderer".to_string(),
        vendor: "Test Vendor".to_string(),
        aliased_line_width_range: Some([1.0, 2.0]),
        aliased_point_size_range: None,
        max_texture_size: None,
        max_viewport_dims: None,
    };

    let noisy1 = injector1.add_webgl_noise(&params);
    let noisy2 = injector2.add_webgl_noise(&params);

    // 相同种子应产生相同of噪声
    assert_eq!(
        noisy1.aliased_line_width_range,
        noisy2.aliased_line_width_range
    );
}

#[test]
fn test_api_noise_injector_creation() {
    let config = NoiseConfig {
        seed: 12345,
        canvas_noise_level: 0.15,
        enable_webgl_noise: true,
        enable_audio_noise: true,
        enable_font_noise: true,
    };

    let injector = ApiNoiseInjector::new(config);

    // testing各个component都可ending with访问
    let _canvas = injector.canvas();
    let _webgl = injector.webgl();
    let _audio = injector.audio();
    let _fonts = injector.fonts();
}

#[test]
fn test_api_noise_injector_defaults() {
    let injector = ApiNoiseInjector::with_defaults();

    let data = vec![255u8; 100];
    let _noisy = injector.canvas().add_noise(&data);

    let fonts = injector.fonts().get_fonts_with_noise(12345);
    assert!(!fonts.is_empty());
}

#[test]
fn test_font_list_consistency() {
    let injector = FontNoiseInjector::new();

    // 同样of种子应该产生同样offontlist
    let fonts1 = injector.get_fonts_with_noise(99999);
    let fonts2 = injector.get_fonts_with_noise(99999);

    assert_eq!(fonts1, fonts2);
}
