use std::collections::HashMap;

use brioche::brioche::{
    project::{resolve_project, DependencyDefinition, ProjectDefinition, Version},
    script::evaluate::evaluate,
};

mod brioche_test;

#[tokio::test]
async fn test_eval_basic() -> anyhow::Result<()> {
    let (brioche, context) = brioche_test::brioche_test().await;

    let project_dir = context.mkdir("myproject").await;
    context
        .write_toml(
            "myproject/brioche.toml",
            &ProjectDefinition {
                dependencies: HashMap::new(),
            },
        )
        .await;

    context
        .write_file(
            "myproject/brioche.bri",
            r#"
                export default () => {
                    return {
                        briocheSerialize: () => {
                            return {
                                type: "directory",
                                entries: {},
                            }
                        },
                    };
                };
            "#,
        )
        .await;

    let project = resolve_project(&brioche, &project_dir).await?;

    let resolved = evaluate(&brioche, &project, "default").await?.value;

    assert_eq!(resolved, brioche_test::lazy_dir_empty());

    Ok(())
}

#[tokio::test]
async fn test_eval_custom_export() -> anyhow::Result<()> {
    let (brioche, context) = brioche_test::brioche_test().await;

    let project_dir = context.mkdir("myproject").await;
    context
        .write_toml(
            "myproject/brioche.toml",
            &ProjectDefinition {
                dependencies: HashMap::new(),
            },
        )
        .await;

    context
        .write_file(
            "myproject/brioche.bri",
            r#"
                export const custom = () => {
                    return {
                        briocheSerialize: () => {
                            return {
                                type: "directory",
                                entries: {},
                            }
                        },
                    };
                };
            "#,
        )
        .await;

    let project = resolve_project(&brioche, &project_dir).await?;

    let resolved = evaluate(&brioche, &project, "custom").await?.value;

    assert_eq!(resolved, brioche_test::lazy_dir_empty());

    Ok(())
}

#[tokio::test]
async fn test_eval_async() -> anyhow::Result<()> {
    let (brioche, context) = brioche_test::brioche_test().await;

    let project_dir = context.mkdir("myproject").await;
    context
        .write_toml(
            "myproject/brioche.toml",
            &ProjectDefinition {
                dependencies: HashMap::new(),
            },
        )
        .await;

    context
        .write_file(
            "myproject/brioche.bri",
            r#"
                export default async () => {
                    return {
                        briocheSerialize: () => {
                            return {
                                type: "directory",
                                entries: {},
                            }
                        },
                    };
                };
            "#,
        )
        .await;

    let project = resolve_project(&brioche, &project_dir).await?;

    let resolved = evaluate(&brioche, &project, "default").await?.value;

    assert_eq!(resolved, brioche_test::lazy_dir_empty());

    Ok(())
}

#[tokio::test]
async fn test_eval_serialize_async() -> anyhow::Result<()> {
    let (brioche, context) = brioche_test::brioche_test().await;

    let project_dir = context.mkdir("myproject").await;
    context
        .write_toml(
            "myproject/brioche.toml",
            &ProjectDefinition {
                dependencies: HashMap::new(),
            },
        )
        .await;

    context
        .write_file(
            "myproject/brioche.bri",
            r#"
                export default async () => {
                    return {
                        briocheSerialize: async () => {
                            return {
                                type: "directory",
                                entries: {},
                            }
                        },
                    };
                };
            "#,
        )
        .await;

    let project = resolve_project(&brioche, &project_dir).await?;

    let resolved = evaluate(&brioche, &project, "default").await?.value;

    assert_eq!(resolved, brioche_test::lazy_dir_empty());

    Ok(())
}

#[tokio::test]
async fn test_eval_import_local() -> anyhow::Result<()> {
    let (brioche, context) = brioche_test::brioche_test().await;

    let project_dir = context.mkdir("myproject").await;
    context
        .write_toml(
            "myproject/brioche.toml",
            &ProjectDefinition {
                dependencies: HashMap::new(),
            },
        )
        .await;

    context
        .write_file(
            "myproject/build.bri",
            r#"
                export const build = async () => {
                    return {
                        briocheSerialize: () => {
                            return {
                                type: "directory",
                                entries: {},
                            }
                        },
                    };
                };
            "#,
        )
        .await;

    context
        .write_file(
            "myproject/brioche.bri",
            r#"
                import { build } from "./build.bri";
                export default async () => {
                    return build();
                };
                "#,
        )
        .await;

    let project = resolve_project(&brioche, &project_dir).await?;

    let resolved = evaluate(&brioche, &project, "default").await?.value;

    assert_eq!(resolved, brioche_test::lazy_dir_empty());

    Ok(())
}

#[tokio::test]
async fn test_eval_import_dep() -> anyhow::Result<()> {
    let (brioche, context) = brioche_test::brioche_test().await;

    let project_dir = context.mkdir("myproject").await;
    context
        .write_toml(
            "myproject/brioche.toml",
            &ProjectDefinition {
                dependencies: HashMap::from_iter([(
                    "foo".into(),
                    DependencyDefinition::Version(Version::Any),
                )]),
            },
        )
        .await;

    context
        .write_file(
            "myproject/brioche.bri",
            r#"
                import { build } from "foo";
                export default async () => {
                    return build();
                };
            "#,
        )
        .await;

    context
        .write_toml(
            "brioche-repo/foo/brioche.toml",
            &ProjectDefinition {
                dependencies: HashMap::new(),
            },
        )
        .await;
    context
        .write_file(
            "brioche-repo/foo/brioche.bri",
            r#"
                export const build = async () => {
                    return {
                        briocheSerialize: () => {
                            return {
                                type: "directory",
                                entries: {},
                            }
                        },
                    };
                };
            "#,
        )
        .await;

    let project = resolve_project(&brioche, &project_dir).await?;

    let resolved = evaluate(&brioche, &project, "default").await?.value;

    assert_eq!(resolved, brioche_test::lazy_dir_empty());

    Ok(())
}