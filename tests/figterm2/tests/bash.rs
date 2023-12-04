use figterm2::Shell;

async fn shell(shell: &str) {
    let mut shell = Shell::init(shell).await.unwrap();

    shell.typed("echo hello world").await.unwrap();
    assert_eq!(Some("echo hello world".into()), shell.buffer().await);
    shell.reset().await.unwrap();

    shell.typed(" \x08").await.unwrap();
    assert_eq!(Some("".into()), shell.buffer().await);
    shell.reset().await.unwrap();

    shell.typed("echo hello world").await.unwrap();
    shell.resize(40, 30).unwrap();
    shell.typed("111").await.unwrap();
    assert_eq!(Some("echo hello world111".into()), shell.buffer().await);
    shell.reset().await.unwrap();
}

#[tokio::test]
async fn bash() {
    shell("bash").await;
}

#[tokio::test]
async fn zsh() {
    shell("zsh").await;
}

#[tokio::test]
async fn fish() {
    shell("fish").await;
}
