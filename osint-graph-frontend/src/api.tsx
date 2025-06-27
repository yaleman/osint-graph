// api.ts
import axios, { type AxiosResponse } from "axios";
import { v4 as uuidv4 } from "uuid";
import type { Project, OSINTNode } from "./types";

const PROJECTS_URL = "/api/v1/projects";
const PROJECT_URL = "/api/v1/project";
const NODE_URL = "/api/v1/node";

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
		creationdate: new Date().valueOf(),
	});
	return response;
};

export const createNode = async (node: OSINTNode): Promise<AxiosResponse<OSINTNode>> => {
	const response = await axios.post<OSINTNode>(NODE_URL, {
		id: node.id,
		project_id: node.project_id,
		node_type: node.node_type,
		display: node.display,
		value: node.value,
		updated: new Date().toISOString(),
		notes: node.notes,
		pos_x: node.pos_x,
		pos_y: node.pos_y,
	});
	return response;
};

export const updateNode = async (node: OSINTNode): Promise<AxiosResponse<OSINTNode>> => {
	const response = await axios.post<OSINTNode>(NODE_URL, {
		...node,
		updated: new Date().toISOString(),
	});
	return response;
};

export const fetchNodesByProject = async (projectId: string): Promise<OSINTNode[]> => {
	const response = await axios.get<OSINTNode[]>(`${PROJECT_URL}/${projectId}/nodes`);
	return response.data;
};

export const createProject = async (projectName: string = "My OSINT Project"): Promise<Project> => {
	const response = await axios.post<Project>(PROJECT_URL, {
		name: projectName,
		id: uuidv4(),
		user: uuidv4(),
		creationdate: Date.now(), // Send timestamp in milliseconds
	});
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
