use std::collections::{BTreeMap, BTreeSet};

use derive_more::Display;
use firefly_client::models::Uri;
use firefly_client::rendering::{Inline, Render};

use crate::ai_agents_teams::compilation::graphl_parsing::Vertex;
use crate::ai_agents_teams::compilation::{Code, Node};
use crate::common::models::RegistryDeploy;
use crate::common::tracing::record_trace;

#[derive(Debug, Clone, Display)]
enum From<'a> {
    #[display("input")]
    Input,
    Channel(&'a str),
}

#[derive(Debug, Clone, Display)]
enum Output<'a> {
    Channel(&'a str),
}

#[derive(Debug, Clone, Render)]
enum NodesTemplate<'a> {
    #[template(path = "ai_agents_teams/nodes/compress.rho")]
    Compress {
        #[template(direct)]
        from: Vec<From<'a>>,

        #[template(direct)]
        output: Output<'a>,

        body: BTreeMap<String, Inline>,
    },

    #[template(path = "ai_agents_teams/nodes/text_model.rho")]
    TextModel {
        #[template(direct)]
        from: From<'a>,

        #[template(direct)]
        output: Output<'a>,
    },

    #[template(path = "ai_agents_teams/nodes/tti_model.rho")]
    TTIModel {
        #[template(direct)]
        from: From<'a>,

        #[template(direct)]
        output: Output<'a>,
    },

    #[template(path = "ai_agents_teams/nodes/tts_model.rho")]
    TTSModel {
        #[template(direct)]
        from: From<'a>,

        #[template(direct)]
        output: Output<'a>,
    },

    #[template(path = "ai_agents_teams/nodes/output.rho")]
    Output {
        #[template(direct)]
        from: From<'a>,
    },
}

#[derive(Debug, Clone, Render)]
#[template(path = "ai_agents_teams/deploy_agents_team.rho")]
struct DeployAgentTeamTemplate<'a> {
    env_uri: Uri,
    version: i64,
    public_key: Vec<u8>,
    sig: Vec<u8>,

    #[template(direct)]
    system_channels: Vec<&'static str>,

    #[template(direct)]
    output_channels: Vec<&'a str>,

    #[template(direct)]
    nodes: Vec<String>,

    #[template(direct)]
    output: bool,
}

fn filter_channels<'a>(from: &[From<'a>]) -> Vec<&'a str> {
    from.iter()
        .filter_map(|f| match f {
            From::Channel(c) => Some(*c),
            From::Input => None,
        })
        .collect()
}

fn get_all_system_channels<'a>(nodes: &BTreeMap<Vertex<'a>, Node<'a>>) -> Vec<&'static str> {
    nodes
        .values()
        .fold(BTreeSet::default(), |mut acc, node| {
            match node {
                Node::Compress { .. } => {}
                Node::TextModel { .. } => {
                    acc.insert("gpt4(`rho:ai:gpt4`)");
                }
                Node::TTIModel { .. } => {
                    acc.insert("dalle3(`rho:ai:dalle3`)");
                }
                Node::TTSModel { .. } => {
                    acc.insert("textToAudio(`rho:ai:textToAudio`)");
                }
            }
            acc
        })
        .into_iter()
        .collect()
}

fn node_output_channel(index: usize) -> String {
    format!("channel{index}Output")
}

fn get_input_for_vertex<'a, 'b>(
    outputs: &'b BTreeMap<&Vertex<'a>, String>,
    vertex: &Vertex<'a>,
) -> From<'b> {
    outputs
        .get(vertex)
        .map_or(From::Input, |s| From::Channel(s))
}

fn get_output_for_vertex<'a, 'b>(
    outputs: &'b BTreeMap<&Vertex<'a>, String>,
    vertex: &'b Vertex<'a>,
) -> Output<'b> {
    outputs
        .get(&vertex)
        .map_or(Output::Channel("devNull"), |s| Output::Channel(s))
}

#[tracing::instrument(
    level = "info",
    skip_all,
    fields(code),
    err(Debug),
    ret(Display, level = "trace")
)]
pub fn render_agent_team(code: Code<'_>, deploy: RegistryDeploy) -> anyhow::Result<String> {
    record_trace!(code);

    let vertex_outputs: BTreeMap<_, _> = code
        .nodes
        .iter()
        .filter_map(|(vertex, node)| node.output().then_some(vertex))
        .enumerate()
        .map(|(index, v)| (v, node_output_channel(index)))
        .collect();

    let output = code
        .output
        .as_ref()
        .map(|v| get_input_for_vertex(&vertex_outputs, &v.from))
        .map(|from| NodesTemplate::Output { from });

    let nodes = code
        .nodes
        .iter()
        .map(|(vertex, node)| match node {
            Node::Compress { from, .. } => NodesTemplate::Compress {
                from: from
                    .iter()
                    .map(|from| get_input_for_vertex(&vertex_outputs, from))
                    .collect(),
                output: get_output_for_vertex(&vertex_outputs, vertex),
                body: from
                    .iter()
                    .map(|from| {
                        (
                            (*from.as_ref()).to_owned(),
                            Inline::from(match get_input_for_vertex(&vertex_outputs, from) {
                                From::Input => From::Input.to_string(),
                                channel @ From::Channel(_) => format!("{channel}Value"),
                            }),
                        )
                    })
                    .collect(),
            },
            Node::TextModel { from, .. } => NodesTemplate::TextModel {
                from: get_input_for_vertex(&vertex_outputs, from),
                output: get_output_for_vertex(&vertex_outputs, vertex),
            },
            Node::TTIModel { from, .. } => NodesTemplate::TTIModel {
                from: get_input_for_vertex(&vertex_outputs, from),
                output: get_output_for_vertex(&vertex_outputs, vertex),
            },
            Node::TTSModel { from, .. } => NodesTemplate::TTSModel {
                from: get_input_for_vertex(&vertex_outputs, from),
                output: get_output_for_vertex(&vertex_outputs, vertex),
            },
        })
        .chain(output)
        .map(Render::render)
        .collect::<Result<_, _>>()?;

    let output_channels = vertex_outputs.values().map(AsRef::as_ref).collect();

    DeployAgentTeamTemplate {
        env_uri: deploy.uri_pub_key.into(),
        version: deploy.version,
        public_key: deploy.uri_pub_key.serialize_uncompressed().into(),
        sig: deploy.signature,
        system_channels: get_all_system_channels(&code.nodes),
        output_channels,
        nodes,
        output: code.output.is_some(),
    }
    .render()
    .map_err(Into::into)
}
