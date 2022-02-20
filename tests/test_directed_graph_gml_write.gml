graph [
  directed 1
  name "test"
  node [
    id 0
    label "node_1"
    rank 0
  ]
  node [
    id 1
    label "node_2"
    rank 1
  ]
  node [
    id 2
    label "node_3"
    rank 1
  ]
  node [
    id 3
    label "node_4"
    rank 2
  ]
  edge [
    source 0
    target 1
    label "1->2"
  ]
  edge [
    source 0
    target 2
    label "1->3"
  ]
  edge [
    source 2
    target 3
    label "3->4"
  ]
]
