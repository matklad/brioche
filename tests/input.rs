use std::{os::unix::prelude::PermissionsExt as _, path::Path, sync::Arc};

use brioche::brioche::{
    value::{CompleteValue, Meta},
    Brioche,
};
use pretty_assertions::assert_eq;

mod brioche_test;

async fn create_input(
    brioche: &Brioche,
    input_path: &Path,
    remove_input: bool,
) -> anyhow::Result<CompleteValue> {
    let value = brioche::brioche::input::create_input(
        brioche,
        brioche::brioche::input::InputOptions {
            input_path,
            remove_input,
            resources_dir: None,
            meta: &Arc::new(Meta::default()),
        },
    )
    .await?;

    Ok(value.value)
}

async fn create_input_with_resources(
    brioche: &Brioche,
    input_path: &Path,
    resources_dir: &Path,
    remove_input: bool,
) -> anyhow::Result<CompleteValue> {
    let value = brioche::brioche::input::create_input(
        brioche,
        brioche::brioche::input::InputOptions {
            input_path,
            remove_input,
            resources_dir: Some(resources_dir),
            meta: &Arc::new(Meta::default()),
        },
    )
    .await?;
    Ok(value.value)
}

#[tokio::test]
async fn test_input_file() -> anyhow::Result<()> {
    let (brioche, context) = brioche_test::brioche_test().await;

    let file_path = context.write_file("hello.txt", b"hello").await;

    let value = create_input(&brioche, &file_path, false).await?;

    assert_eq!(
        value,
        brioche_test::file(brioche_test::blob(&brioche, b"hello").await, false)
    );
    assert!(file_path.is_file());

    Ok(())
}

#[tokio::test]
async fn test_input_executable_file() -> anyhow::Result<()> {
    let (brioche, context) = brioche_test::brioche_test().await;

    let file_path = context.write_file("hello.txt", b"hello").await;
    let mut permissions = tokio::fs::metadata(&file_path)
        .await
        .expect("failed to get metadata")
        .permissions();
    permissions.set_mode(0o755);
    tokio::fs::set_permissions(&file_path, permissions)
        .await
        .expect("failed to set permissions");

    let value = create_input(&brioche, &file_path, false).await?;

    assert_eq!(
        value,
        brioche_test::file(brioche_test::blob(&brioche, b"hello").await, true)
    );
    assert!(file_path.is_file());

    Ok(())
}

#[tokio::test]
async fn test_input_symlink() -> anyhow::Result<()> {
    let (brioche, context) = brioche_test::brioche_test().await;

    let symlink_path = context.write_symlink("/foo", "foo").await;

    let value = create_input(&brioche, &symlink_path, false).await?;

    assert_eq!(value, brioche_test::symlink("/foo"));
    assert!(symlink_path.is_symlink());

    Ok(())
}

#[tokio::test]
async fn test_input_empty_dir() -> anyhow::Result<()> {
    let (brioche, context) = brioche_test::brioche_test().await;

    let dir_path = context.mkdir("test").await;

    let value = create_input(&brioche, &dir_path, false).await?;

    assert_eq!(value, brioche_test::dir_empty());
    assert!(dir_path.is_dir());

    Ok(())
}

#[tokio::test]
async fn test_input_dir() -> anyhow::Result<()> {
    let (brioche, context) = brioche_test::brioche_test().await;

    let dir_path = context.mkdir("test").await;
    context.write_file("test/hello/hi.txt", b"hello").await;
    context.mkdir("test/empty").await;
    context.write_symlink("hello/hi.txt", "test/link").await;

    let value = create_input(&brioche, &dir_path, false).await?;

    assert_eq!(
        value,
        brioche_test::dir([
            (
                "hello",
                brioche_test::dir([(
                    "hi.txt",
                    brioche_test::file(brioche_test::blob(&brioche, b"hello").await, false)
                ),])
            ),
            ("empty", brioche_test::dir_empty()),
            ("link", brioche_test::symlink("hello/hi.txt"))
        ])
    );
    assert!(dir_path.is_dir());

    Ok(())
}

#[tokio::test]
async fn test_input_remove_original() -> anyhow::Result<()> {
    let (brioche, context) = brioche_test::brioche_test().await;

    let dir_path = context.mkdir("test").await;
    context.write_file("test/hello/hi.txt", b"hello").await;
    context.mkdir("test/empty").await;
    context.write_symlink("hello/hi.txt", "test/link").await;

    let value = create_input(&brioche, &dir_path, true).await?;

    assert_eq!(
        value,
        brioche_test::dir([
            (
                "hello",
                brioche_test::dir([(
                    "hi.txt",
                    brioche_test::file(brioche_test::blob(&brioche, b"hello").await, false)
                ),])
            ),
            ("empty", brioche_test::dir_empty()),
            ("link", brioche_test::symlink("hello/hi.txt"))
        ])
    );
    assert!(!dir_path.is_dir());

    Ok(())
}

#[tokio::test]
async fn test_input_dir_treat_pack_normally() -> anyhow::Result<()> {
    let (brioche, context) = brioche_test::brioche_test().await;

    let dir_path = context.mkdir("test").await;

    let mut packed_file = b"test".to_vec();
    brioche_pack::inject_pack(
        &mut packed_file,
        &brioche_pack::Pack {
            program: b"test".into(),
            interpreter: None,
        },
    )?;

    context.write_file("test/hi", &packed_file).await;
    context
        .write_file("test/brioche-pack.d/test", b"test")
        .await;

    let value = create_input(&brioche, &dir_path, false).await?;

    assert_eq!(
        value,
        brioche_test::dir([
            (
                "hi",
                brioche_test::file(brioche_test::blob(&brioche, &packed_file).await, false)
            ),
            (
                "brioche-pack.d",
                brioche_test::dir([(
                    "test",
                    brioche_test::file(brioche_test::blob(&brioche, b"test").await, false)
                ),])
            ),
        ])
    );
    assert!(dir_path.is_dir());

    Ok(())
}

#[tokio::test]
async fn test_input_dir_use_resource_dir() -> anyhow::Result<()> {
    let (brioche, context) = brioche_test::brioche_test().await;

    let dir_path = context.mkdir("test").await;
    let resources_dir = context.mkdir("resources").await;

    let mut packed_file = b"test".to_vec();
    brioche_pack::inject_pack(
        &mut packed_file,
        &brioche_pack::Pack {
            program: b"test".into(),
            interpreter: None,
        },
    )?;

    context.write_file("test/hi", &packed_file).await;
    context.write_file("resources/test", b"test").await;

    let value = create_input_with_resources(&brioche, &dir_path, &resources_dir, false).await?;

    assert_eq!(
        value,
        brioche_test::dir([(
            "hi",
            brioche_test::file_with_resources(
                brioche_test::blob(&brioche, &packed_file).await,
                false,
                brioche_test::dir_value([(
                    "test",
                    brioche_test::file(brioche_test::blob(&brioche, b"test").await, false),
                )]),
            )
        ),])
    );
    assert!(dir_path.is_dir());

    Ok(())
}

#[tokio::test]
async fn test_input_dir_with_symlink_resources() -> anyhow::Result<()> {
    let (brioche, context) = brioche_test::brioche_test().await;

    let dir_path = context.mkdir("test").await;
    let resources_dir = context.mkdir("resources").await;

    let mut packed_file = b"test".to_vec();
    brioche_pack::inject_pack(
        &mut packed_file,
        &brioche_pack::Pack {
            program: b"test".into(),
            interpreter: None,
        },
    )?;

    context.write_file("test/hi", &packed_file).await;
    context.write_symlink("test_target", "resources/test").await;
    context.write_file("resources/test_target", b"test").await;

    let value = create_input_with_resources(&brioche, &dir_path, &resources_dir, false).await?;

    assert_eq!(
        value,
        brioche_test::dir([(
            "hi",
            brioche_test::file_with_resources(
                brioche_test::blob(&brioche, &packed_file).await,
                false,
                brioche_test::dir_value([
                    ("test", brioche_test::symlink(b"test_target")),
                    (
                        "test_target",
                        brioche_test::file(brioche_test::blob(&brioche, b"test").await, false)
                    ),
                ]),
            )
        ),])
    );
    assert!(dir_path.is_dir());

    Ok(())
}

#[tokio::test]
async fn test_input_dir_broken_symlink() -> anyhow::Result<()> {
    let (brioche, context) = brioche_test::brioche_test().await;

    let dir_path = context.mkdir("test").await;
    let resources_dir = context.mkdir("resources").await;

    let mut packed_file = b"test".to_vec();
    brioche_pack::inject_pack(
        &mut packed_file,
        &brioche_pack::Pack {
            program: b"test".into(),
            interpreter: None,
        },
    )?;

    context.write_file("test/hi", &packed_file).await;
    context.write_symlink("test_target", "resources/test").await;

    let value = create_input_with_resources(&brioche, &dir_path, &resources_dir, false).await?;

    assert_eq!(
        value,
        brioche_test::dir([(
            "hi",
            brioche_test::file_with_resources(
                brioche_test::blob(&brioche, &packed_file).await,
                false,
                brioche_test::dir_value([("test", brioche_test::symlink(b"test_target"))]),
            )
        ),])
    );
    assert!(dir_path.is_dir());

    Ok(())
}

#[tokio::test]
async fn test_input_dir_with_dir_resources() -> anyhow::Result<()> {
    let (brioche, context) = brioche_test::brioche_test().await;

    let dir_path = context.mkdir("test").await;
    let resources_dir = context.mkdir("resources").await;

    let mut packed_file = b"test".to_vec();
    brioche_pack::inject_pack(
        &mut packed_file,
        &brioche_pack::Pack {
            program: b"test".into(),
            interpreter: None,
        },
    )?;

    context.write_file("test/hi", &packed_file).await;
    context.write_file("resources/test/hi", b"test").await;
    context
        .write_symlink("../test_target", "resources/test/target")
        .await;
    context.write_file("resources/test_target", b"test").await;

    let value = create_input_with_resources(&brioche, &dir_path, &resources_dir, false).await?;

    assert_eq!(
        value,
        brioche_test::dir([(
            "hi",
            brioche_test::file_with_resources(
                brioche_test::blob(&brioche, &packed_file).await,
                false,
                brioche_test::dir_value([
                    (
                        "test",
                        brioche_test::dir([
                            (
                                "hi",
                                brioche_test::file(
                                    brioche_test::blob(&brioche, b"test").await,
                                    false
                                )
                            ),
                            ("target", brioche_test::symlink(b"../test_target")),
                        ])
                    ),
                    (
                        "test_target",
                        brioche_test::file(brioche_test::blob(&brioche, b"test").await, false)
                    ),
                ]),
            )
        ),])
    );
    assert!(dir_path.is_dir());

    Ok(())
}

#[tokio::test]
async fn test_input_dir_omits_unused_resources() -> anyhow::Result<()> {
    let (brioche, context) = brioche_test::brioche_test().await;

    let dir_path = context.mkdir("test").await;
    let resources_dir = context.mkdir("resources").await;

    let mut packed_file = b"test".to_vec();
    brioche_pack::inject_pack(
        &mut packed_file,
        &brioche_pack::Pack {
            program: b"test".into(),
            interpreter: None,
        },
    )?;

    context.write_file("test/hi", &packed_file).await;
    context.write_file("resources/test", "hello").await;

    // Not referenced by any pack
    context.write_file("resources/unused.txt", "other").await;

    let value = create_input_with_resources(&brioche, &dir_path, &resources_dir, false).await?;

    assert_eq!(
        value,
        brioche_test::dir([(
            "hi",
            brioche_test::file_with_resources(
                brioche_test::blob(&brioche, &packed_file).await,
                false,
                brioche_test::dir_value([(
                    "test",
                    brioche_test::file(brioche_test::blob(&brioche, b"hello").await, false),
                )]),
            )
        ),])
    );
    assert!(dir_path.is_dir());

    Ok(())
}