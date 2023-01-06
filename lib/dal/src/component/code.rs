use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use telemetry::prelude::*;

use crate::attribute::value::AttributeValue;
use crate::attribute::value::AttributeValueError;
use crate::component::ComponentResult;
use crate::{
    AttributeReadContext, CodeLanguage, CodeView, ComponentError, ComponentId, DalContext,
    StandardModel, WsEvent, WsPayload,
};
use crate::{Component, SchemaVariant};
use crate::{RootPropChild, WsEventResult};

#[derive(Deserialize, Debug)]
struct CodeGenerationEntry {
    pub code: Option<String>,
    pub format: Option<String>,
}

impl Component {
    #[instrument(skip_all)]
    pub async fn list_code_generated(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<Vec<CodeView>> {
        let component = Self::get_by_id(ctx, &component_id)
            .await?
            .ok_or(ComponentError::NotFound(component_id))?;
        let schema_variant = component
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(component_id))?;

        // Prepare to assemble code views and access the "/root/code" prop tree.
        let mut code_views: Vec<CodeView> = Vec::new();
        let code_map_implicit_internal_provider =
            SchemaVariant::find_root_child_implicit_internal_provider(
                ctx,
                *schema_variant.id(),
                RootPropChild::Code,
            )
            .await?;
        let code_map_attribute_read_context = AttributeReadContext {
            internal_provider_id: Some(*code_map_implicit_internal_provider.id()),
            component_id: Some(component_id),
            ..AttributeReadContext::default()
        };
        let code_map_attribute_value =
            AttributeValue::find_for_context(ctx, code_map_attribute_read_context)
                .await?
                .ok_or(AttributeValueError::NotFoundForReadContext(
                    code_map_attribute_read_context,
                ))?;
        let maybe_code_map_value = code_map_attribute_value.get_value(ctx).await?;

        // If the map has been populated, we need to see if there are code views to generate.
        if let Some(code_map_value) = maybe_code_map_value {
            let code_map: HashMap<String, CodeGenerationEntry> =
                serde_json::from_value(code_map_value)?;

            for entry in code_map.values() {
                // When a new code gen function is craeted the code/format entries will not yet be
                // set, so just ignore them in the loop here. Function return value type checking
                // should ensure that the executed function does not unset these itself.
                if entry.format.is_none() || entry.code.is_none() {
                    continue;
                }

                // Safe unwraps because of the above check
                let format = entry.format.as_ref().unwrap();
                let code = entry.code.as_ref().unwrap();

                let language = if format.is_empty() {
                    CodeLanguage::Unknown
                } else {
                    CodeLanguage::try_from(format.to_owned())?
                };

                // TODO(nick): determine how we handle empty code generation or generation in
                // progress. Maybe we never need to? Just re-run?
                let code = if code.is_empty() {
                    None
                } else {
                    Some(code.clone())
                };

                code_views.push(CodeView::new(language, code));
            }
        }
        Ok(code_views)
    }
}

// NOTE(nick): consider moving this somewhere else.
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeGeneratedPayload {
    component_id: ComponentId,
}

// NOTE(nick): consider moving this somewhere else.
impl WsEvent {
    pub async fn code_generated(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::CodeGenerated(CodeGeneratedPayload { component_id }),
        )
        .await
    }
}
