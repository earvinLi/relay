/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

//! This module is responsible to build a single project. It does not handle
//! watch mode or other state.

mod apply_transforms;
mod build_ir;
mod build_schema;
mod generate_artifacts;
mod validate;
mod write_artifacts;

use crate::compiler::AstSets;
use crate::compiler_state::CompilerState;
use crate::config::{Config, ConfigProject};
use crate::errors::BuildProjectError;
use build_ir::BuildIRResult;
use common::Timer;
use graphql_ir::{Program, Sources, ValidationError};

pub async fn build_project(
    compiler_state: &CompilerState,
    config: &Config,
    project_config: &ConfigProject,
    ast_sets: &AstSets,
    sources: &Sources<'_>,
) -> Result<(), BuildProjectError> {
    // Construct a schema instance including project specific extensions.
    let schema = Timer::time(format!("build_schema {}", project_config.name), || {
        build_schema::build_schema(compiler_state, project_config)
    });

    // Build a type aware IR.
    let BuildIRResult {
        ir,
        base_fragment_names,
    } = Timer::time(format!("build_ir {}", project_config.name), || {
        add_error_sources(
            build_ir::build_ir(project_config, &schema, ast_sets),
            sources,
        )
    })?;

    // Turn the IR into a base Program.
    let program = Timer::time(format!("build_program {}", project_config.name), || {
        Program::from_definitions(&schema, ir)
    });

    // Call validation rules that go beyond type checking.
    Timer::time(format!("validate {}", project_config.name), || {
        add_error_sources(validate::validate(&program), sources)
    })?;

    // Apply various chains of transforms to create a set of output programs.
    let programs = Timer::time(format!("apply_transforms {}", project_config.name), || {
        apply_transforms::apply_transforms(&program, &base_fragment_names)
    });

    // Generate code and persist text to produce output artifacts in memory.
    let artifacts_timer = Timer::start(format!("generate_artifacts {}", project_config.name));
    let artifacts = generate_artifacts::generate_artifacts(project_config, &programs).await?;
    artifacts_timer.stop();

    // Write the generated artifacts to disk. This step is separte from
    // generating artifacts to avoid partial writes in case of errors as
    // much as possible.
    Timer::time(format!("write_artifacts {}", project_config.name), || {
        write_artifacts::write_artifacts(config, project_config, &artifacts)
    })?;

    println!(
        "[{}] documents: {} reader, {} normalization, {} operation",
        project_config.name,
        programs.reader.document_count(),
        programs.normalization.document_count(),
        programs.operation_text.document_count()
    );

    Ok(())
}

fn add_error_sources<T>(
    result: Result<T, Vec<ValidationError>>,
    sources: &Sources<'_>,
) -> Result<T, BuildProjectError> {
    result.map_err(|validation_errors| BuildProjectError::ValidationErrors {
        errors: validation_errors
            .into_iter()
            .map(|error| error.with_sources(sources))
            .collect(),
    })
}
