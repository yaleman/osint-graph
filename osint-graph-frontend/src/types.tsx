// types.ts
export interface Project {
	id: string;
	name: string;
	user: string;
	creationdate: Date;
	last_updated?: Date;
	tags: string[];
	description?: string;
	// Add other fields as necessary
}

export interface OSINTNode {
	id: string;
	project_id: string;
	node_type: string;
	display: string;
	value: string;
	updated: string;
	notes?: string;
	pos_x: number;
	pos_y: number;
	attachments: string[];
}

export interface NodeLink {
	id: string;
	left: string;
	right: string;
	project_id: string;
	linktype: "Omni" | "Directional";
}

export interface Attachment {
	id: string;
	node_id: string;
	filename: string;
	content_type: string;
	size: number;
	created: string;
}

export interface ProjectExport {
	project: Project;
	nodes: OSINTNode[];
	nodelinks: NodeLink[];
	exported_at: string;
	version: string;
	attachments: Attachment[];
}

export const NodeTypeInfo: Record<
	string,
	{
		label: string;
		defaultDisplay: string;
		color: string;
		syncedvalue?: boolean;
	}
> = {
	person: {
		label: "Person",
		defaultDisplay: "Name",
		color: "#3b82f6",
		syncedvalue: true,
	},
	domain: { label: "Domain", defaultDisplay: "Domain", color: "#f59e0b" },
	ip: { label: "IP Address", defaultDisplay: "Address", color: "#ef4444" },
	phone: { label: "Phone", defaultDisplay: "Number", color: "#8b5cf6" },
	email: { label: "Email", defaultDisplay: "Address", color: "#ec4899" },
	url: { label: "URL", defaultDisplay: "Link", color: "#06b6d4" },
	image: {
		label: "Image",
		defaultDisplay: "Filename",
		color: "#10b981",
		syncedvalue: true,
	},
	location: { label: "Location", defaultDisplay: "Address", color: "#84cc16" },
	organisation: {
		label: "Organisation",
		defaultDisplay: "Name",
		color: "#f97316",
		syncedvalue: true,
	},
	document: {
		label: "Document",
		defaultDisplay: "Filename",
		color: "#6b7280",
		syncedvalue: true,
	},
	currency: {
		label: "Currency",
		defaultDisplay: "Amount",
		color: "#c7c400ff",
		syncedvalue: false,
	},
};

export const hasSyncedValue = (nodeType: string): boolean => {
	const info = NodeTypeInfo[nodeType];
	return info?.syncedvalue ?? false;
};

export const getNodeColor = (nodeType: string): string => {
	const typeInfo = NodeTypeInfo[nodeType] ?? { color: "#6b7280" };
	return typeInfo.color;
};
