/*
    # The Occlusion Engine
    This function will:
    - take a series of OcclusionRules from config
    - occlude specified resources from the graph
    - if there are relationships for [left_hand_resource -> occlude_resource -> right_hand_resource] 
    then the engine will remove the old relationships and create a new relationship for [left_hand_resource -> right_hand_resource]
*/

// TODO: 
// [√] load occlusions
// [ ] occlude nodes
// [ ] occlude relationships

// QUESTION: do we:
// 1) remove resources/relationships 
// or 
// 2) simply mark occluded items with a visibility=false flag?

pub fn occlude() -> OccludedPolicyGraph {
}

#[cfg(test)]
mod tests {

    #[test]
    fn execute_occlusion() {
        let graph = Graph::new();
        let result = super::occlude();
        assert_eq!()
    }
}

// export function mergeRoleRelationships(roleRelationshipPair) {
//     if (roleRelationshipPair[0].length === 0 || roleRelationshipPair[1].length === 0) {
//         return null;
//     }

//     const newEdges = roleRelationshipPair[0].map(rel => 
//         ({ id: `${rel.id}c`, sources: [ roleRelationshipPair[1][0].sources[0] ], targets: [ rel.targets[0] ] })
//     );
//     const sourcesEdgesToBeDeleted = roleRelationshipPair[0].map(rel => rel.id);

//     return {
//         nodesToBeDeleted: roleRelationshipPair[1][0].targets[0],
//         edgesToBeDeleted: [...sourcesEdgesToBeDeleted, roleRelationshipPair[1][0].id],
//         newEdges
//     }
// }


// const children = generateNodes(data.resources);
// const edges = generateEdges(data.relationships, children);

// const iamRoles = children.filter(child => child.type === 'aws_iam_role');
// const roleRelationships = iamRoles.map(role => {
//     const list = edges.filter(edge => edge.sources[0] === role.id || edge.targets[0] === role.id);
//     return R.partition(edge => edge.sources[0] === role.id, list);
// });

// const results = roleRelationships.map(mergeRoleRelationships).filter(_=>_);
