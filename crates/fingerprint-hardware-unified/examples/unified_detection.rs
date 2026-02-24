//! Example demonstrating the unified hardware fingerprint detection
//!
//! This example shows how to use the consolidated hardware fingerprint
//! detection module instead of individual hardware modules.

use fingerprint_hardware_unified::{
    HardwareFingerprint,
    HardwareError,
    DeviceProfile,
};
use tokio;

#[tokio::main]
async fn main() -> Result<(), HardwareError> {
    println!("=== Unified Hardware Fingerprint Detection ===");
    
    // Create and configure hardware fingerprint collector
    let mut hardware_fp = HardwareFingerprint::new();
    
    // Collect all enabled hardware fingerprints
    println!("Collecting hardware fingerprints...");
    hardware_fp.collect_all().await?;
    
    // Display results
    println!("\nHardware Fingerprint Results:");
    println!("ID: {}", hardware_fp.id);
    
    #[cfg(feature = "canvas")]
    if let Some(canvas) = &hardware_fp.canvas {
        println!("Canvas Renderer: {}", canvas.canvas_renderer);
        println!("WebGL Vendor: {}", canvas.webgl_vendor);
        println!("WebGL Renderer: {}", canvas.webgl_renderer);
    }
    
    #[cfg(feature = "audio")]
    if let Some(audio) = &hardware_fp.audio {
        println!("Audio Context Type: {}", audio.audio_context_type);
        println!("Sample Rate: {} Hz", audio.sample_rate);
        println!("Channel Count: {}", audio.channel_count);
    }
    
    #[cfg(feature = "fonts")]
    if let Some(fonts) = &hardware_fp.fonts {
        println!("Available Fonts: {} detected", fonts.available_fonts.len());
    }
    
    #[cfg(feature = "storage")]
    if let Some(storage) = &hardware_fp.storage {
        println!("Local Storage Available: {}", storage.local_storage_available);
        println!("Session Storage Available: {}", storage.session_storage_available);
        println!("IndexedDB Available: {}", storage.indexed_db_available);
    }
    
    #[cfg(feature = "webrtc")]
    if let Some(webrtc) = &hardware_fp.webrtc {
        println!("Media Devices: {} detected", webrtc.media_devices.len());
        println!("ICE Candidates: {} detected", webrtc.ice_candidates.len());
    }
    
    // Display device profile
    println!("\nDevice Profile:");
    println!("GPU Vendor: {}", hardware_fp.device_profile.gpu_vendor);
    println!("CPU Cores: {}", hardware_fp.device_profile.cpu_cores);
    println!("Memory: {} GB", hardware_fp.device_profile.memory_gb);
    println!("Screen Resolution: {}x{}", 
             hardware_fp.device_profile.screen_resolution.0,
             hardware_fp.device_profile.screen_resolution.1);
    println!("Platform: {}", hardware_fp.device_profile.platform);
    println!("Confidence Score: {:.2}", hardware_fp.device_profile.confidence_score);
    
    println!("\n=== Detection Complete ===");
    Ok(())
}