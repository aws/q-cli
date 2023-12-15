use std::io::Write as _;

use amzn_codewhisperer_streaming_client::types::{
    ChatMessage,
    ChatResponseStream,
    ChatTriggerType,
    ConversationState,
    EditorState,
    Position,
    ProgrammingLanguage,
    TextDocument,
    UserInputMessage,
    UserInputMessageContext,
};
use eyre::Result;
use fig_api_client::ai::cw_streaming_client;

pub async fn chat() -> Result<()> {
    let client = cw_streaming_client().await;

    // clear screen
    print!("\x1B[2J\x1B[1;1H");

    println!(
        "Q > Hi, I'm Amazon Q. I can answer your software development questions. Ask me to explain, debug, or optimize your code."
    );
    println!();

    loop {
        print!("User > ");
        std::io::stdout().flush().unwrap();
        let user_input = std::io::stdin().lines().next().unwrap().unwrap();

        println!();

        let mut res = client
            .generate_assistant_response()
            .conversation_state(
                ConversationState::builder()
                    .current_message(ChatMessage::UserInputMessage(
                        UserInputMessage::builder()
                            .content(user_input)
                            .user_input_message_context(
                                UserInputMessageContext::builder()
                                    .editor_state(
                                        EditorState::builder()
                                            .document(
                                                TextDocument::builder()
                                                    .text("#!/bin/bash\n\n")
                                                    .relative_file_path("file.sh")
                                                    .programming_language(
                                                        ProgrammingLanguage::builder()
                                                            .language_name("bash")
                                                            .build()
                                                            .unwrap(),
                                                    )
                                                    
                                                    .build()
                                                    .unwrap(),
                                            )
                                            .cursor_state(
                                                amzn_codewhisperer_streaming_client::types::CursorState::Position(
                                                    Position::builder().line(2).character(0).build().unwrap(),
                                                ),
                                            )
                                            .build(),
                                    )
                                    .build(),
                            )
                            .user_intent(amzn_codewhisperer_streaming_client::types::UserIntent::ImproveCode)
                            .build()
                            .unwrap(),
                    ))
                    .chat_trigger_type(ChatTriggerType::Manual)
                    .build()
                    .unwrap(),
            )
            .send()
            .await
            .unwrap();

        while let Ok(Some(a)) = res.generate_assistant_response_response.recv().await {
            match a {
                ChatResponseStream::MessageMetadataEvent(response) => {
                    // println!("{:?}", response.conversation_id());
                    print!("Q > ");
                    std::io::stdout().flush().unwrap();
                },
                ChatResponseStream::AssistantResponseEvent(response) => {
                    print!("{}", response.content());
                },
                ChatResponseStream::FollowupPromptEvent(response) => {
                    let followup = response.followup_prompt().unwrap();
                    println!("content: {}", followup.content());
                    println!("intent: {:?}", followup.user_intent());
                },
                ChatResponseStream::CodeReferenceEvent(_) => {},
                ChatResponseStream::SupplementaryWebLinksEvent(_) => {},
                _ => {},
            }
        }
        println!();
        println!();
    }

    Ok(())
}
