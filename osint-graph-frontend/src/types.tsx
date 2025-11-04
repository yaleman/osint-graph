// types.ts
export interface Project {
	id: string;
	name: string;
	user: string;
	creationdate: Date;
	tags: string[];
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
	linktype: 'Omni' | 'Directional';
}

export const NodeTypeInfo: Record<string, { label: string; defaultDisplay: string }> = {
	person: { label: 'Person', defaultDisplay: 'Name' },
	domain: { label: 'Domain', defaultDisplay: 'Domain' },
	ip: { label: 'IP Address', defaultDisplay: 'Address' },
	phone: { label: 'Phone', defaultDisplay: 'Number' },
	email: { label: 'Email', defaultDisplay: 'Address' },
	url: { label: 'URL', defaultDisplay: 'Link' },
	image: { label: 'Image', defaultDisplay: 'Filename' },
	location: { label: 'Location', defaultDisplay: 'Address' },
	organization: { label: 'Organization', defaultDisplay: 'Name' },
	document: { label: 'Document', defaultDisplay: 'Filename' },
};
