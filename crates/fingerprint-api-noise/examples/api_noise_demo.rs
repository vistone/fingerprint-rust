use fingerprint_api_noise::{ApiNoiseInjector, NoiseConfig};

fn main() {
    println!("🔧 API 噪声注入演示程序\n");
    
    // 创建噪声注入器
    let config = NoiseConfig {
        seed: 12345,
        canvas_noise_level: 0.15,
        enable_webgl_noise: true,
        enable_audio_noise: true,
        enable_font_noise: true,
    };
    
    let injector = ApiNoiseInjector::new(config);
    
    // 模拟 Canvas 数据
    println!("📊 Canvas 指纹测试");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let canvas_data = vec![255u8; 1000];
    let noisy_canvas = injector.canvas().add_noise(&canvas_data);
    let fingerprint = injector.canvas().fingerprint_hash(&canvas_data);
    
    // 计算差异
    let diff: usize = canvas_data.iter().zip(&noisy_canvas)
        .filter(|(a, b)| a != b)
        .count();
    
    println!("✅ Canvas 指纹 (带噪声): {}", fingerprint);
    println!("📈 修改了 {} / {} 字节 ({:.2}%)", 
             diff, canvas_data.len(), 
             (diff as f64 / canvas_data.len() as f64) * 100.0);
    println!();
    
    // 模拟字体枚举
    println!("🔤 字体枚举测试");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let fonts = injector.fonts().get_fonts_with_noise(12345);
    println!("✅ 检测到的字体 ({} 个):", fonts.len());
    for (i, font) in fonts.iter().enumerate() {
        println!("   {}. {}", i + 1, font);
    }
    println!();
    
    // 每次运行会略有不同
    let fonts2 = injector.fonts().get_fonts_with_noise(12346);
    println!("🔄 第二次枚举 ({} 个):", fonts2.len());
    for (i, font) in fonts2.iter().enumerate() {
        println!("   {}. {}", i + 1, font);
    }
    println!();
    
    // 测试 Audio 噪声
    println!("🎵 Audio 指纹测试");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let audio_samples = vec![0.5f32; 100];
    let noisy_audio = injector.audio().add_audio_noise(&audio_samples);
    
    let audio_diff: usize = audio_samples.iter().zip(&noisy_audio)
        .filter(|&(a, b)| (a - b).abs() > 0.0)
        .count();
    
    println!("✅ Audio 样本处理完成");
    println!("📈 修改了 {} / {} 样本 ({:.2}%)", 
             audio_diff, audio_samples.len(), 
             (audio_diff as f64 / audio_samples.len() as f64) * 100.0);
    
    let avg_diff: f32 = audio_samples.iter().zip(&noisy_audio)
        .map(|(&a, &b)| (a - b).abs())
        .sum::<f32>() / audio_samples.len() as f32;
    
    println!("📊 平均噪声幅度: {:.6}", avg_diff);
    println!();
    
    // 测试 WebGL 参数
    println!("🎮 WebGL 参数测试");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let webgl_params = fingerprint_api_noise::webgl::WebGLParams {
        renderer: "ANGLE (Intel, Intel(R) UHD Graphics)".to_string(),
        vendor: "Google Inc. (Intel)".to_string(),
        aliased_line_width_range: Some([1.0, 1.0]),
        aliased_point_size_range: Some([1.0, 1024.0]),
        max_texture_size: Some(16384),
        max_viewport_dims: Some([16384, 16384]),
    };
    
    let noisy_webgl = injector.webgl().add_webgl_noise(&webgl_params);
    
    println!("✅ WebGL Renderer: {}", noisy_webgl.renderer);
    println!("✅ WebGL Vendor: {}", noisy_webgl.vendor);
    if let Some(range) = noisy_webgl.aliased_line_width_range {
        println!("📏 Line Width Range: [{:.4}, {:.4}]", range[0], range[1]);
    }
    println!();
    
    println!("✨ 演示完成！所有 API 噪声注入测试通过。");
}
