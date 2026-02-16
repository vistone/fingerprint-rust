//! Example: AI-Generated Content Detection
//!
//! Demonstrates detection and analysis of AI-generated text content,
//! including perplexity analysis, burstiness metrics, and model attribution.

use fingerprint_ai_models::content_detection::{detect_ai_content, PatternType};

fn main() {
    println!("=== AI-Generated Content Detection Demo ===\n");

    // Example 1: Clearly AI-generated text (GPT-style)
    println!("Example 1: AI-Generated Text (GPT-style)");
    println!("----------------------------------------");
    let ai_text = "Artificial intelligence has revolutionized numerous sectors of modern society. \
                   It's important to note that machine learning algorithms continue to advance at an unprecedented pace. \
                   Furthermore, the implications for businesses and individuals are profound. \
                   Moreover, we must delve into the ethical considerations surrounding these technologies. \
                   In conclusion, AI represents both opportunities and challenges for humanity.";
    
    analyze_and_display(ai_text, "AI-Generated (GPT)");
    println!();

    // Example 2: Human-written text
    println!("Example 2: Human-Written Text");
    println!("-----------------------------");
    let human_text = "Hey! So I was thinking about AI the other day... \
                      It's kinda crazy how much it's changed, right? \
                      Like, my phone can basically read my mind now lol. \
                      But honestly? I'm not sure if that's good or bad. \
                      What do you think?";
    
    analyze_and_display(human_text, "Human-Written");
    println!();

    // Example 3: Claude-style text (more structured)
    println!("Example 3: AI-Generated Text (Claude-style)");
    println!("-------------------------------------------");
    let claude_text = "The integration of artificial intelligence into modern workflows presents several key advantages. \
                       Firstly, it enhances operational efficiency through automation. \
                       Secondly, it enables data-driven decision-making processes. \
                       Thirdly, it facilitates scalability across organizational structures. \
                       Therefore, organizations must carefully consider implementation strategies.";
    
    analyze_and_display(claude_text, "AI-Generated (Claude)");
    println!();

    // Example 4: Short, concise text (Gemini-style)
    println!("Example 4: AI-Generated Text (Gemini-style)");
    println!("-------------------------------------------");
    let gemini_text = "AI transforms industries rapidly. Key benefits include automation, accuracy, and scalability. \
                       Challenges involve ethics and job displacement. Implementation requires careful planning.";
    
    analyze_and_display(gemini_text, "AI-Generated (Gemini)");
    println!();

    // Example 5: Mixed/edited text
    println!("Example 5: Mixed Human-AI Text");
    println!("------------------------------");
    let mixed_text = "I've been researching AI lately, and it's important to note that the technology has come a long way. \
                      But tbh, I'm still skeptical about some of the claims. \
                      Furthermore, the ethical implications are significant - we can't just ignore that!";
    
    analyze_and_display(mixed_text, "Mixed Text");
    println!();

    // Comparison summary
    println!("=== Analysis Summary ===");
    println!("Metrics Explanation:");
    println!("  • Perplexity: Lower = more predictable = more AI-like");
    println!("  • Burstiness: Lower = more uniform = more AI-like");
    println!("  • Vocab Richness: Moderate (0.3-0.6) is typical for AI");
    println!("  • Confidence: Overall likelihood text is AI-generated");
    println!("\nDetection Patterns:");
    println!("  • RepetitiveStructure: Similar sentence beginnings");
    println!("  • FormalLanguage: Excessive formal connectors");
    println!("  • AiPhrases: Characteristic AI expressions");
    println!("  • UniformSentenceLength: Consistent sentence lengths");
    println!();
}

fn analyze_and_display(text: &str, label: &str) {
    let result = detect_ai_content(text);
    
    println!("Text: \"{}...\"", &text[..text.len().min(60)]);
    println!("\n📊 Detection Result:");
    println!("  AI-Generated: {} (Confidence: {:.1}%)", 
             if result.is_ai_generated { "✓ YES" } else { "✗ NO" },
             result.confidence * 100.0);
    
    println!("\n📈 Metrics:");
    println!("  • Perplexity:      {:.3} {}", 
             result.perplexity,
             if result.perplexity < 0.3 { "(Very AI-like)" } 
             else if result.perplexity < 0.5 { "(AI-like)" }
             else { "(Human-like)" });
    
    println!("  • Burstiness:      {:.3} {}", 
             result.burstiness,
             if result.burstiness < 0.3 { "(Very uniform - AI-like)" }
             else if result.burstiness < 0.6 { "(Moderate)" }
             else { "(Varied - Human-like)" });
    
    println!("  • Vocab Richness:  {:.3}", result.vocabulary_richness);
    
    println!("\n📝 Content Info:");
    println!("  • Characters:      {}", result.metadata.char_count);
    println!("  • Words:           {}", result.metadata.word_count);
    println!("  • Sentences:       {}", result.metadata.sentence_count);
    println!("  • Avg Sent Length: {:.1} words", result.metadata.avg_sentence_length);
    println!("  • Unique Words:    {}", result.metadata.unique_words);
    
    if !result.model_probabilities.is_empty() {
        println!("\n🤖 Model Attribution:");
        let mut probs: Vec<_> = result.model_probabilities.iter().collect();
        probs.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
        
        for (model, prob) in probs {
            if *prob > 0.15 {
                let bar_len = (*prob * 30.0) as usize;
                let bar = "█".repeat(bar_len);
                println!("  {:8} [{:3.0}%] {}", 
                         model.to_uppercase(), 
                         prob * 100.0, 
                         bar);
            }
        }
    }
    
    if !result.patterns.is_empty() {
        println!("\n🔍 Detected Patterns:");
        for pattern in &result.patterns {
            let icon = match pattern.pattern_type {
                PatternType::AiPhrases => "💬",
                PatternType::RepetitiveStructure => "🔁",
                PatternType::FormalLanguage => "📜",
                _ => "•",
            };
            println!("  {} {:?}: {} (confidence: {:.0}%)", 
                     icon,
                     pattern.pattern_type, 
                     pattern.description,
                     pattern.confidence * 100.0);
        }
    }
}
