use rag_pdf_dataset::rag::{Retriever, RetrievedChunk, PromptBuilder};
use serde_json::json;

#[test]
fn test_prompt_building_with_retrieved_chunks() {
    let query = "What is machine learning?";

    let chunks = vec![
        RetrievedChunk {
            score: 0.95,
            payload: json!({
                "content": "Machine learning is a subset of artificial intelligence."
            }),
        },
        RetrievedChunk {
            score: 0.87,
            payload: json!({
                "content": "It enables computers to learn from data without being explicitly programmed."
            }),
        },
    ];

    let prompt = PromptBuilder::build_prompt(query, &chunks);

    assert!(prompt.contains("Machine learning"));
    assert!(prompt.contains("artificial intelligence"));
    assert!(prompt.contains("What is machine learning?"));
    assert!(prompt.contains("Context:"));
}

#[test]
fn test_prompt_building_with_empty_chunks() {
    let query = "Test query";
    let chunks = vec![];

    let prompt = PromptBuilder::build_prompt(query, &chunks);

    assert!(prompt.contains("Test query"));
    assert!(prompt.contains("Context:"));
}

#[test]
fn test_prompt_structure() {
    let chunks = vec![RetrievedChunk {
        score: 0.9,
        payload: json!({
            "content": "Sample content"
        }),
    }];

    let prompt = PromptBuilder::build_prompt("Query?", &chunks);

    // Should have context section
    assert!(prompt.contains("Context:"));
    // Should have question section
    assert!(prompt.contains("Question"));
    // Should contain the query
    assert!(prompt.contains("Query?"));
}
