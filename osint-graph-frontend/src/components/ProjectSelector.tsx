import { useEffect, useState } from "react";
import { fetchProjects } from "../api";
import type { Project } from "../types";

interface ProjectSelectorProps {
	currentProject: Project | null;
	onProjectChange: (projectId: string) => void;
	onCreateNew: () => void;
	setShowProjectManagement: (show: boolean) => void;
}

export function ProjectSelector({
	currentProject,
	onProjectChange,
	onCreateNew,
	setShowProjectManagement,
}: ProjectSelectorProps) {
	const [isOpen, setIsOpen] = useState(false);
	const [projects, setProjects] = useState<Project[]>([]);
	const [loading, setLoading] = useState(false);

	const loadProjects = async () => {
		if (loading === false) {
			setLoading(true);
			try {
				const projectList = await fetchProjects();
				setProjects(projectList);
			} catch (error) {
				console.error("Failed to load projects:", error);
				setProjects([]);
			} finally {
				setLoading(false);
			}
		}
	};

	// biome-ignore lint: lint/correctness/useExhaustiveDependencies "adding loadProjects causes infinite loop"
	useEffect(() => {
		if (isOpen) {
			loadProjects();
		}
	}, [isOpen]);

	return (
		<>
			{/* Backdrop to capture clicks outside the dropdown */}
			{isOpen && (
				<button
					type="button"
					className="click-away-backdrop"
					onClick={() => setIsOpen(false)}
					onKeyDown={(e) => {
						if (e.key === "Escape") {
							setIsOpen(false);
						}
					}}
					aria-label="Close project selector"
				/>
			)}
			<div
				style={{ position: "fixed", top: "10px", left: "10px", zIndex: 1000 }}
			>
				<div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
					<button
						type="button"
						onClick={() => setIsOpen(!isOpen)}
						className="project-selector btn"
					>
						<span>📁</span>
						<span>{currentProject?.name ?? "No Project"}</span>
						<span className="smol">{isOpen ? "▲" : "▼"}</span>
					</button>

					<button
						type="button"
						onClick={onCreateNew}
						className="btn btn-primary"
						title="Create New Project"
					>
						+ New
					</button>
					<button
						type="button"
						onClick={() => setShowProjectManagement(true)}
						className="btn btn-primary"
						title="Project Settings"
					>
						⚙️ Project Settings
					</button>
				</div>

				{isOpen && (
					<div className="project-selector-dropdown">
						{loading ? (
							<div className="project-selector-noprojects">
								Loading projects...
							</div>
						) : projects.length === 0 ? (
							<div className="project-selector-noprojects">
								No projects found
							</div>
						) : (
							<div>
								{projects.map((project, index) => (
									<div
										role="menuitem"
										tabIndex={index}
										key={project.id}
										onKeyDown={() => {}}
										onClick={() => {
											onProjectChange(project.id);
											setIsOpen(false);
										}}
										className={`project-selector-base ${currentProject?.id === project.id ? "project-selector-selected" : ""}`}
										onMouseEnter={(e) => {
											if (currentProject?.id !== project.id) {
												e.currentTarget.style.background = "#f9fafb";
											}
										}}
										onMouseLeave={(e) => {
											if (currentProject?.id !== project.id) {
												e.currentTarget.style.background = "white";
											}
										}}
									>
										<div>{project.name}</div>
										<div className="project-selector-subhead">{project.id}</div>
									</div>
								))}
							</div>
						)}
					</div>
				)}
			</div>
		</>
	);
}
