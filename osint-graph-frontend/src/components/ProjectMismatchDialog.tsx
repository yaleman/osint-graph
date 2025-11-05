import { useState } from "react";
import type { Project } from "../types";
import "../osint-graph.css";

interface ProjectMismatchDialogProps {
	onCreateNew: () => void;
	onProjectSelect: (projectId: string) => void;
	projects: Project[];
}

export function ProjectMismatchDialog({
	onCreateNew,
	onProjectSelect,
	projects,
}: ProjectMismatchDialogProps) {
	const [showProjectList, setShowProjectList] = useState(false);

	if (showProjectList) {
		return (
			<div className="dialog-backdrop">
				<div className="dialog-container">
					<h2 className="dialog-title">Select a Project</h2>
					<p className="dialog-description">
						Choose a project to continue working on:
					</p>

					<div className="project-list">
						{projects.map((project) => (
							<div
								key={project.id}
								className="project-list-item"
								onClick={() => onProjectSelect(project.id)}
								onKeyDown={(e) => {
									if (e.key === "Enter" || e.key === " ") {
										onProjectSelect(project.id);
									}
								}}
								role="button"
								tabIndex={0}
							>
								<div className="project-list-item-name">{project.name}</div>
								<div className="project-list-item-id">{project.id}</div>
								{project.description && (
									<div className="project-list-item-description">
										{project.description}
									</div>
								)}
							</div>
						))}
					</div>

					<button
						type="button"
						onClick={() => setShowProjectList(false)}
						className="btn btn-secondary"
					>
						Back
					</button>
				</div>
			</div>
		);
	}

	return (
		<div className="dialog-backdrop">
			<div className="dialog-container">
				<h2 className="dialog-title">Project Not Found</h2>
				<p className="dialog-description">
					The project stored in your browser doesn't exist in the backend. This
					might happen if the database was reset or the project was deleted.
				</p>

				<div className="dialog-question">
					<p className="dialog-question-text">What would you like to do?</p>
				</div>

				<div className="dialog-buttons">
					<button
						type="button"
						onClick={onCreateNew}
						className="btn btn-primary"
					>
						Create New Project
					</button>

					{projects.length > 0 && (
						<button
							type="button"
							onClick={() => setShowProjectList(true)}
							className="btn btn-secondary"
						>
							Select Existing Project ({projects.length} available)
						</button>
					)}
				</div>

				{projects.length === 0 && (
					<p className="dialog-no-projects">
						No existing projects found in the backend.
					</p>
				)}
			</div>
		</div>
	);
}
