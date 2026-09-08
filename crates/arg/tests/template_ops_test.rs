// SPDX-License-Identifier: MIT OR Apache-2.0
//! `template_ops_test` — 5 UT (per DD §10.1.1 UT-14..UT-18).

use star_arg::models::template::{
    all_templates, chain, hierarchical, hub_and_spoke, mesh, review_council, TemplateCategory,
    TemplateId,
};
use star_arg::ops::template_ops::build_mesh_edges;

#[test]
fn ut14_template_hub_and_spoke() {
    let t = hub_and_spoke();
    assert_eq!(t.id, TemplateId::HubAndSpoke);
    assert_eq!(t.min_agents, 5);
    assert_eq!(t.max_agents, 5);
    assert_eq!(t.edges.len(), 4);
    assert_eq!(t.category, TemplateCategory::Standard);
    assert_eq!(t.edges[0].from_index, 0);
    assert_eq!(t.edges[0].to_index, 1);
}

#[test]
fn ut15_template_mesh_edge_count() {
    let t = mesh();
    let edges_5 = build_mesh_edges(5);
    assert_eq!(edges_5.len(), 10); // C(5,2) = 10
    let edges_10 = build_mesh_edges(10);
    assert_eq!(edges_10.len(), 45); // C(10,2) = 45
    assert_eq!(t.min_agents, 3);
    assert_eq!(t.max_agents, 10);
    assert_eq!(t.category, TemplateCategory::HighDensity);
}

#[test]
fn ut16_template_chain() {
    let t = chain();
    assert_eq!(t.id, TemplateId::Chain);
    assert_eq!(t.min_agents, 4);
    assert_eq!(t.edges.len(), 3);
    assert_eq!(t.category, TemplateCategory::Pipeline);
}

#[test]
fn ut17_template_hierarchical_three_layers() {
    let t = hierarchical();
    assert_eq!(t.id, TemplateId::Hierarchical);
    assert_eq!(t.min_agents, 9);
    assert_eq!(t.max_agents, 9);
    // 1 Lead→2 Sub + 2 Sub→6 Worker = 8 edges
    assert_eq!(t.edges.len(), 8);
    assert_eq!(t.category, TemplateCategory::Management);
}

#[test]
fn ut18_template_review_council() {
    let t = review_council();
    assert_eq!(t.id, TemplateId::ReviewCouncil);
    assert_eq!(t.min_agents, 4);
    assert_eq!(t.edges.len(), 3);
    assert_eq!(t.category, TemplateCategory::Decision);
    // 5 templates registered in total.
    assert_eq!(all_templates().len(), 5);
}
