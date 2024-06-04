// api.ts
import axios, { type AxiosResponse } from "axios";
import { v4 as uuidv4 } from "uuid";
import type { Project } from "./types";

const PROJECTS_URL = "/api/v1/projects";
const PROJECT_URL = "/api/v1/project";

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
