pub type GraphIndex = usize;

// A simple DAG implementation without deletion.
#[derive(Debug)]
pub struct DAG<T> {
    data: Vec<T>,                // data for each node
    edges: Vec<Vec<GraphIndex>>, // each node's outgoing edges
}

impl<T> Default for DAG<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> DAG<T> {
    pub fn new() -> DAG<T> {
        DAG {
            data: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn get(&self, from: GraphIndex) -> Option<&T> {
        self.data.get(from)
    }

    pub fn node(&mut self, value: T) -> GraphIndex {
        let index = self.data.len();
        self.data.push(value);
        self.edges.push(Vec::new());
        index
    }

    pub fn connect(&mut self, from: GraphIndex, to: GraphIndex) -> bool {
        let maxi = self.data.len() - 1;
        if from > maxi || to > maxi {
            return false;
        }
        self.edges[from].push(to);
        true
    }

    pub fn successors(&self, from: GraphIndex) -> Option<Successors> {
        if from > self.data.len() - 1 {
            return None;
        }
        Some(Successors {
            data: self.edges[from].to_vec(),
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Successors {
    data: Vec<GraphIndex>,
}

impl Iterator for Successors {
    type Item = GraphIndex;

    fn next(&mut self) -> Option<GraphIndex> {
        self.data.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_dag_simple() {
        #[derive(Debug, PartialEq, Eq)]
        enum C {
            First,
            Second,
            Third,
            Fourth,
        }

        let mut dag: DAG<C> = DAG::new();
        let i0 = dag.node(C::First);
        let i1 = dag.node(C::Second);
        let i2 = dag.node(C::Third);
        let i3 = dag.node(C::Fourth);
        dag.connect(i0, i1);
        dag.connect(i0, i2);
        dag.connect(i2, i3);

        assert_eq!(&C::Fourth, dag.get(i3).unwrap());
        assert_eq!(&C::Third, dag.get(i2).unwrap());
        assert_eq!(&C::Second, dag.get(i1).unwrap());
        assert_eq!(&C::First, dag.get(i0).unwrap());

        assert_eq!(Some(Successors { data: vec![] }), dag.successors(i3));
        assert_eq!(Some(Successors { data: vec![i3] }), dag.successors(i2));
        assert_eq!(Some(Successors { data: vec![] }), dag.successors(i1));
        assert_eq!(Some(Successors { data: vec![i1, i2] }), dag.successors(i0));
    }
}
