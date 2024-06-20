// types.ts
export interface Project {
	id: string;
	name: string;
	user: string;
	creationdate: Date;
	// Add other fields as necessary
}

export interface Dataset {
	nodes: NodeData[];
	edges: [string, string][];
	clusters: Cluster[];
	tags: Tag[];
}

export interface FiltersState {
	clusters: Record<string, boolean>;
	tags: Record<string, boolean>;
}

export interface NodeData {
	key: string;
	label: string;
	tag: string;
	URL: string;
	cluster: string;
	x: number;
	y: number;
}

export interface Cluster {
	key: string;
	color: string;
	clusterLabel: string;
}

export interface Tag {
	key: string;
	image: string;
}
