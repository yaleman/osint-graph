// api.ts
import axios, { type AxiosResponse } from "axios";
import { v4 as uuidv4 } from "uuid";
import type { NodeLink, OSINTNode, Project, ProjectExport } from "./types";

const PROJECTS_URL = "/api/v1/projects";
const PROJECT_URL = "/api/v1/project";
const NODE_URL = "/api/v1/node";
const NODELINK_URL = "/api/v1/nodelink";

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
): Promise<ProjectExport> => {
	const response = await axios.get<ProjectExport>(
		`${PROJECT_URL}/${projectId}/export`,
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
