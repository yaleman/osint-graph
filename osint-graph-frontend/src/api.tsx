// api.ts
import axios, { type AxiosResponse } from "axios";
import { v4 as uuidv4 } from "uuid";
import type {
	Attachment,
	NodeLink,
	OSINTNode,
	Project,
	ProjectExport,
	SearchResult,
} from "./types";

const PROJECTS_URL = "/api/v1/projects";
const PROJECT_URL = "/api/v1/project";
const NODE_URL = "/api/v1/node";
const ATTACHMENT_URL = "/api/v1/attachment";
const NODELINK_URL = "/api/v1/nodelink";
const SEARCH_URL = "/api/v1/search";

// Authentication callback that will be set by the AuthContext
let authFailureCallback: (() => void) | null = null;

export function setAuthFailureCallback(callback: () => void) {
	authFailureCallback = callback;
}

// Configure axios interceptor to detect authentication failures
axios.interceptors.response.use(
	(response) => {
		// Check if we were redirected to a login page
		// The response.request.responseURL contains the final URL after redirects
		const finalUrl = response.request.responseURL as string;
		if (
			finalUrl &&
			(finalUrl.includes("/admin/login") || finalUrl.endsWith("/admin/login"))
		) {
			// We were redirected to login, trigger auth failure
			if (authFailureCallback) {
				authFailureCallback();
			}
			return Promise.reject(new Error("Authentication required"));
		}
		return response;
	},
	(error) => {
		// Handle 401 Unauthorized responses
		if (error.response?.status === 401) {
			if (authFailureCallback) {
				authFailureCallback();
			}
		}
		return Promise.reject(error);
	},
);

export const fetchProjects = async (): Promise<Project[]> => {
	const response = await axios.get<Project[]>(PROJECTS_URL);
	return response.data;
};

export const newProject = async (): Promise<AxiosResponse<Project, string>> => {
	// create a new project where the id is a new uuid4 and the name is "New Project"
	const response = await axios.post<Project>(PROJECT_URL, {
		name: "New Project",
		id: uuidv4(),
		user: uuidv4(),
		creationdate: new Date().toISOString,
		tags: [],
	});
	return response;
};

export const createNode = async (
	node: OSINTNode,
): Promise<AxiosResponse<OSINTNode>> => {
	const nodeData = {
		id: node.id,
		project_id: node.project_id,
		node_type: node.node_type,
		display: node.display,
		value: node.value,
		updated: node.updated,
		pos_x: node.pos_x,
		pos_y: node.pos_y,
		attachments: node.attachments ?? [],
		...(node.notes && { notes: node.notes }),
	};
	const response = await axios.post<OSINTNode>(NODE_URL, nodeData);
	return response;
};

export const updateNode = async (
	node: OSINTNode,
): Promise<AxiosResponse<OSINTNode>> => {
	const response = await axios.put<OSINTNode>(`${NODE_URL}/${node.id}`, {
		...node,
		updated: new Date().toISOString(),
	});
	return response;
};

export const fetchNodesByProject = async (
	projectId: string,
): Promise<OSINTNode[]> => {
	const response = await axios.get<OSINTNode[]>(
		`${PROJECT_URL}/${projectId}/nodes`,
	);
	return response.data;
};

export const createProject = async (
	projectName: string = "My OSINT Project",
): Promise<Project> => {
	const response = await axios.post<Project>(PROJECT_URL, {
		name: projectName,
		id: uuidv4(),
		user: uuidv4(),
		creationdate: new Date().toISOString(),
		tags: [],
	});
	return response.data;
};

export const getProject = async (
	projectId: string,
): Promise<Project | null> => {
	try {
		const response = await axios.get<Project>(`${PROJECT_URL}/${projectId}`);
		return response.data;
	} catch (error) {
		if (axios.isAxiosError(error) && error.response?.status === 404) {
			return null;
		}
		throw error;
	}
};

export const createNodeLink = async (
	nodelink: NodeLink,
): Promise<AxiosResponse<NodeLink>> => {
	const response = await axios.post<NodeLink>(NODELINK_URL, nodelink);
	return response;
};

export const fetchNodeLinksByProject = async (
	projectId: string,
): Promise<NodeLink[]> => {
	const response = await axios.get<NodeLink[]>(
		`${PROJECT_URL}/${projectId}/nodelinks`,
	);
	return response.data;
};

export const deleteNode = async (nodeId: string): Promise<void> => {
	await axios.delete(`${NODE_URL}/${nodeId}`);
};

export const deleteNodeLink = async (nodelinkId: string): Promise<void> => {
	await axios.delete(`${NODELINK_URL}/${nodelinkId}`);
};

export const updateProject = async (
	projectId: string,
	project: Project,
): Promise<Project> => {
	const response = await axios.put<Project>(
		`${PROJECT_URL}/${projectId}`,
		project,
	);
	return response.data;
};

export const deleteProject = async (projectId: string): Promise<void> => {
	await axios.delete(`${PROJECT_URL}/${projectId}`);
};

export const exportProject = async (
	projectId: string,
	includeAttachments: boolean,
): Promise<ProjectExport> => {
	const params =
		(includeAttachments ?? false) ? { include_attachments: "true" } : {};
	const response = await axios.get<ProjectExport>(
		`${PROJECT_URL}/${projectId}/export`,
		{ params },
	);
	return response.data;
};

export const exportProjectMermaid = async (
	projectId: string,
): Promise<string> => {
	const response = await axios.get<string>(
		`${PROJECT_URL}/${projectId}/export/mermaid`,
		{ responseType: "text" as "json" }, // Trick TypeScript while telling axios to expect text
	);
	return response.data;
};

/** Upload a file attachment to a node */
export const uploadAttachment = async (
	nodeId: string,
	file: File,
): Promise<Attachment> => {
	const formData = new FormData();
	formData.append("file", file);

	// Don't set Content-Type - let axios set it with the correct boundary
	const response = await axios.post<Attachment>(
		`${NODE_URL}/${nodeId}/attachment`,
		formData,
	);
	return response.data;
};

/** Download a file attachment */
export const downloadAttachment = async (
	attachmentId: string,
): Promise<Blob> => {
	const response = await axios.get(`${ATTACHMENT_URL}/${attachmentId}`, {
		responseType: "blob",
	});
	return response.data;
};

/** Delete a file attachment */
export const deleteAttachment = async (attachmentId: string): Promise<void> => {
	await axios.delete(`${ATTACHMENT_URL}/${attachmentId}`);
};

/** Update an attachment (e.g., move to a different node) */
export const updateAttachment = async (
	attachmentId: string,
	nodeId: string,
): Promise<Attachment> => {
	const response = await axios.patch<Attachment>(
		`${ATTACHMENT_URL}/${attachmentId}`,
		{ node_id: nodeId },
	);
	return response.data;
};

/** List all attachments for a node */
export const listAttachments = async (
	nodeId: string,
): Promise<Attachment[]> => {
	const response = await axios.get<Attachment[]>(
		`${NODE_URL}/${nodeId}/attachments`,
	);
	return response.data;
};

/** Takes the project list and sends back a list */
export function projectLis(projects: Project[]) {
	return (
		<ul>
			{projects.map((project) => {
				return <li key={project.id}>{project.name}</li>;
			})}
		</ul>
	);
}

/** Search across all projects for nodes matching the query */
export const searchGlobal = async (query: string): Promise<SearchResult[]> => {
	if (!query.trim()) {
		return [];
	}
	const response = await axios.get<SearchResult[]>(SEARCH_URL, {
		params: { q: query },
	});
	return response.data;
};
