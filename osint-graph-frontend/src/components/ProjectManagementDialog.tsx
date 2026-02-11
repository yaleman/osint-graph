import type React from "react";
import { useEffect, useId, useRef, useState } from "react";
import toast from "react-hot-toast";
import {
	deleteProject,
	exportProject,
	exportProjectMermaid,
	importProject,
	updateProject,
} from "../api";
import type {
	ImportMode,
	Project,
	ProjectExport,
	ProjectImportResult,
} from "../types";
import "../osint-graph.css";
import { MermaidViewerDialog } from "./MermaidViewerDialog";

interface ProjectManagementDialogProps {
	isOpen: boolean;
	onClose: () => void;
	currentProject: Project | null;
	onProjectUpdate: (project: Project) => void;
	onProjectDelete: () => void;
	onProjectImport: (result: ProjectImportResult) => Promise<void>;
}

type TabType = "general" | "export" | "import" | "delete";
const PROJECT_SETTINGS_TABS: ReadonlyArray<{ id: TabType; label: string }> = [
	{ id: "general", label: "General" },
	{ id: "export", label: "Export" },
	{ id: "import", label: "Import" },
	{ id: "delete", label: "Delete" },
];

export const ProjectManagementDialog: React.FC<
	ProjectManagementDialogProps
> = ({
	isOpen,
	onClose,
	currentProject,
	onProjectUpdate,
	onProjectDelete,
	onProjectImport,
}) => {
	const [activeTab, setActiveTab] = useState<TabType>("general");
	const [loading, setLoading] = useState(false);

	// General tab state
	const [projectName, setProjectName] = useState("");
	const [projectDescription, setProjectDescription] = useState("");
	const [projectTags, setProjectTags] = useState<string[]>([]);
	const [newTag, setNewTag] = useState("");

	// Delete tab state
	const [deleteConfirmName, setDeleteConfirmName] = useState("");

	// Export tab state
	const [exportData, setExportData] = useState<ProjectExport | null>(null);

	// Import tab state
	const [importData, setImportData] = useState<ProjectExport | null>(null);
	const [importFileName, setImportFileName] = useState("");
	const [importMode, setImportMode] = useState<ImportMode>("new");
	const [importDragActive, setImportDragActive] = useState(false);
	const [overwriteConfirmed, setOverwriteConfirmed] = useState(false);

	// Mermaid state
	const [mermaidViewerOpen, setMermaidViewerOpen] = useState(false);
	const [mermaidCode, setMermaidCode] = useState("");

	const idProjectName = useId();
	const idProjectDescription = useId();
	const idProjectTags = useId();
	const idDeleteConfirmName = useId();
	const idImportFile = useId();
	const idImportMode = useId();
	const idOverwriteConfirm = useId();
	const idDialogTitle = useId();
	const idTabPrefix = useId();
	const idTabPanelPrefix = useId();
	const tabRefs = useRef<Record<TabType, HTMLButtonElement | null>>({
		general: null,
		export: null,
		import: null,
		delete: null,
	});

	// Initialize form with current project data
	useEffect(() => {
		if (currentProject) {
			setProjectName(currentProject.name);
			setProjectDescription(currentProject.description ?? "");
			setProjectTags(currentProject.tags ?? []);
		}
	}, [currentProject]);

	if (!isOpen || !currentProject) return null;

	const handleSaveGeneral = async () => {
		if (!projectName.trim()) {
			toast.error("Project name cannot be empty");
			return;
		}

		setLoading(true);
		try {
			const updatedProject = await updateProject(currentProject.id, {
				id: currentProject.id,
				user: currentProject.user,
				creationdate: currentProject.creationdate,
				name: projectName,
				tags: projectTags,
				...(projectDescription.trim() && { description: projectDescription }),
			});
			onProjectUpdate(updatedProject);
			toast.success("Project updated successfully");
			onClose(); // Close the dialog after successful save
		} catch (error) {
			console.error("Failed to update project:", error);
			toast.error("Failed to update project");
		} finally {
			setLoading(false);
		}
	};

	const handleAddTag = () => {
		if (newTag.trim() && !projectTags.includes(newTag.trim())) {
			setProjectTags([...projectTags, newTag.trim()]);
			setNewTag("");
		}
	};

	const handleRemoveTag = (tagToRemove: string) => {
		setProjectTags(projectTags.filter((tag) => tag !== tagToRemove));
	};

	const handleExport = async () => {
		setLoading(true);
		try {
			const data = await exportProject(currentProject.id, true);
			setExportData(data);

			// Create download
			const blob = new Blob([JSON.stringify(data, null, 2)], {
				type: "application/json",
			});
			const url = URL.createObjectURL(blob);
			const a = document.createElement("a");
			a.href = url;
			const timestamp = new Date()
				.toISOString()
				.replace(/[:.]/g, "-")
				.split("T")[0];
			a.download = `${currentProject.name.replace(/[^a-z0-9]/gi, "_")}_${timestamp}.json`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);

			toast.success("Project exported successfully");
		} catch (error) {
			console.error("Failed to export project:", error);
			toast.error("Failed to export project");
		} finally {
			setLoading(false);
		}
	};

	const handleDelete = async () => {
		if (deleteConfirmName !== currentProject.name) {
			toast.error("Project name does not match");
			return;
		}

		if (
			!window.confirm(
				"Are you sure you want to delete this project? This action cannot be undone.",
			)
		) {
			return;
		}

		setLoading(true);
		try {
			await deleteProject(currentProject.id);
			onProjectDelete();
			// Don't call onClose() - onProjectDelete already handles closing
		} catch (error) {
			console.error("Failed to delete project:", error);
			toast.error("Failed to delete project");
		} finally {
			setLoading(false);
		}
	};

	const handleExportMermaid = async () => {
		setLoading(true);
		try {
			const mermaidDiagram = await exportProjectMermaid(currentProject.id);

			// Create download
			const blob = new Blob([mermaidDiagram], {
				type: "text/plain",
			});
			const url = URL.createObjectURL(blob);
			const a = document.createElement("a");
			a.href = url;
			const timestamp = new Date()
				.toISOString()
				.replace(/[:.]/g, "-")
				.split("T")[0];
			a.download = `${currentProject.name.replace(/[^a-z0-9]/gi, "_")}_${timestamp}.mmd`;
			document.body.appendChild(a);
			a.click();
			document.body.removeChild(a);
			URL.revokeObjectURL(url);

			toast.success("Mermaid diagram exported successfully");
		} catch (error) {
			console.error("Failed to export Mermaid diagram:", error);
			toast.error("Failed to export Mermaid diagram");
		} finally {
			setLoading(false);
		}
	};

	const handleViewMermaid = async () => {
		setLoading(true);
		try {
			const mermaidDiagram = await exportProjectMermaid(currentProject.id);
			setMermaidCode(mermaidDiagram);
			setMermaidViewerOpen(true);
			// Update URL to include mermaid view
			window.location.hash = `project=${currentProject.id}&view=mermaid`;
		} catch (error) {
			console.error("Failed to load Mermaid diagram:", error);
			toast.error("Failed to load Mermaid diagram");
		} finally {
			setLoading(false);
		}
	};

	const parseImportPayload = (raw: string): ProjectExport => {
		let parsed: unknown;
		try {
			parsed = JSON.parse(raw);
		} catch (error) {
			throw new Error(
				`Invalid JSON: ${error instanceof Error ? error.message : "unknown parse error"}`,
			);
		}

		const payload = parsed as Partial<ProjectExport>;
		if (
			!payload.project ||
			!payload.nodes ||
			!payload.nodelinks ||
			!payload.attachments
		) {
			throw new Error(
				"Invalid import file: expected project, nodes, nodelinks, and attachments fields",
			);
		}

		return payload as ProjectExport;
	};

	const loadImportFile = async (file: File) => {
		try {
			const contents = await file.text();
			const payload = parseImportPayload(contents);
			setImportData(payload);
			setImportFileName(file.name);
			toast.success("Import file loaded");
		} catch (error) {
			console.error("Failed to parse import file:", error);
			setImportData(null);
			setImportFileName("");
			toast.error(
				error instanceof Error ? error.message : "Failed to parse import file",
			);
		}
	};

	const handleImportFileChange: React.ChangeEventHandler<HTMLInputElement> = (
		event,
	) => {
		const file = event.target.files?.[0];
		if (!file) return;
		void loadImportFile(file);
	};

	const handleImportDrop: React.DragEventHandler<HTMLDivElement> = (event) => {
		event.preventDefault();
		setImportDragActive(false);
		const file = event.dataTransfer.files?.[0];
		if (!file) return;
		void loadImportFile(file);
	};

	const handleImport = async () => {
		if (!importData) {
			toast.error("Select an import JSON file first");
			return;
		}

		if (importMode === "overwrite" && !overwriteConfirmed) {
			toast.error("Confirm overwrite before importing");
			return;
		}

		setLoading(true);
		try {
			const targetProjectId =
				importMode === "new" ? undefined : currentProject.id;
			const result = await importProject(
				importData,
				importMode,
				targetProjectId,
			);
			await onProjectImport(result);
			setImportData(null);
			setImportFileName("");
			setOverwriteConfirmed(false);
		} catch (error) {
			console.error("Failed to import project:", error);
			toast.error("Failed to import project");
		} finally {
			setLoading(false);
		}
	};

	const focusTab = (tab: TabType) => {
		tabRefs.current[tab]?.focus();
	};

	const activateTabByOffset = (currentTab: TabType, offset: number) => {
		const currentIndex = PROJECT_SETTINGS_TABS.findIndex(
			(tab) => tab.id === currentTab,
		);
		if (currentIndex === -1) {
			return;
		}

		const nextIndex =
			(currentIndex + offset + PROJECT_SETTINGS_TABS.length) %
			PROJECT_SETTINGS_TABS.length;
		const nextTab = PROJECT_SETTINGS_TABS[nextIndex];
		if (!nextTab) {
			return;
		}

		setActiveTab(nextTab.id);
		focusTab(nextTab.id);
	};

	const activateEdgeTab = (edge: "first" | "last") => {
		const nextTab =
			edge === "first"
				? PROJECT_SETTINGS_TABS[0]
				: PROJECT_SETTINGS_TABS[PROJECT_SETTINGS_TABS.length - 1];
		if (!nextTab) {
			return;
		}
		setActiveTab(nextTab.id);
		focusTab(nextTab.id);
	};

	const handleTabKeyDown = (
		event: React.KeyboardEvent<HTMLButtonElement>,
		currentTab: TabType,
	) => {
		switch (event.key) {
			case "ArrowRight":
			case "ArrowDown":
				event.preventDefault();
				activateTabByOffset(currentTab, 1);
				break;
			case "ArrowLeft":
			case "ArrowUp":
				event.preventDefault();
				activateTabByOffset(currentTab, -1);
				break;
			case "Home":
				event.preventDefault();
				activateEdgeTab("first");
				break;
			case "End":
				event.preventDefault();
				activateEdgeTab("last");
				break;
			default:
				break;
		}
	};

	const handleDialogKeyDown: React.KeyboardEventHandler<HTMLDivElement> = (
		event,
	) => {
		if (event.key === "Escape") {
			event.preventDefault();
			onClose();
		}
	};

	return (
		<div className="dialog-backdrop">
			<button
				type="button"
				className="click-away-backdrop"
				onClick={onClose}
				aria-label="Close project settings dialog"
			/>
			<div
				role="dialog"
				aria-modal="true"
				aria-labelledby={idDialogTitle}
				className="dialog-container"
				onKeyDown={handleDialogKeyDown}
			>
				{/* Header */}
				<div className="dialog-header">
					<h2 id={idDialogTitle} className="dialog-title">
						Project Settings
					</h2>
					<button
						type="button"
						onClick={onClose}
						className="btn btn-transparent"
					>
						×
					</button>
				</div>

				{/* Tabs */}
				<div
					className="dialog-tabs"
					role="tablist"
					aria-label="Project settings tabs"
				>
					{PROJECT_SETTINGS_TABS.map((tab) => {
						const isActive = activeTab === tab.id;
						return (
							<button
								key={tab.id}
								id={`${idTabPrefix}-${tab.id}`}
								type="button"
								role="tab"
								aria-selected={isActive}
								aria-controls={`${idTabPanelPrefix}-${tab.id}`}
								tabIndex={isActive ? 0 : -1}
								className={`dialog-tab ${isActive ? "active" : ""}`}
								ref={(el) => {
									tabRefs.current[tab.id] = el;
								}}
								onClick={() => setActiveTab(tab.id)}
								onKeyDown={(event) => handleTabKeyDown(event, tab.id)}
							>
								{tab.label}
							</button>
						);
					})}
				</div>

				{/* Tab Content */}
				<div className="dialog-content">
					{/* General Tab */}
					{activeTab === "general" && (
						<div
							role="tabpanel"
							id={`${idTabPanelPrefix}-general`}
							aria-labelledby={`${idTabPrefix}-general`}
							className="dialog-tabpanel"
						>
							<div className="form-group">
								<label className="form-label" htmlFor={idProjectName}>
									Project Name *
								</label>
								<input
									type="text"
									id={idProjectName}
									value={projectName}
									onChange={(e) => setProjectName(e.target.value)}
									className="form-input"
									placeholder="Enter project name"
								/>
							</div>

							<div className="form-group">
								<label className="form-label" htmlFor={idProjectDescription}>
									Description
								</label>
								<textarea
									id={idProjectDescription}
									value={projectDescription}
									onChange={(e) => setProjectDescription(e.target.value)}
									className="form-textarea"
									placeholder="Enter project description"
								/>
							</div>

							<div className="form-group">
								<label className="form-label" htmlFor={idProjectTags}>
									Tags
								</label>
								<div className="tags-container">
									{projectTags.map((tag) => (
										<div key={tag} className="tag">
											{tag}
											<button
												type="button"
												onClick={() => handleRemoveTag(tag)}
												className="btn tag-remove-button"
											>
												×
											</button>
										</div>
									))}
								</div>
								<div className="tag-input-group">
									<input
										type="text"
										id={idProjectTags}
										value={newTag}
										onChange={(e) => setNewTag(e.target.value)}
										onKeyPress={(e) => {
											if (e.key === "Enter") {
												e.preventDefault();
												handleAddTag();
											}
										}}
										className="tag-input"
										placeholder="Add a tag"
									/>
									<button
										type="button"
										onClick={handleAddTag}
										className="btn btn-primary"
									>
										Add
									</button>
								</div>
							</div>

							<button
								type="button"
								onClick={handleSaveGeneral}
								disabled={loading || !projectName.trim()}
								className="btn btn-primary"
							>
								{loading ? "Saving..." : "Save Changes"}
							</button>
						</div>
					)}

					{/* Export Tab */}
					{activeTab === "export" && (
						<div
							role="tabpanel"
							id={`${idTabPanelPrefix}-export`}
							aria-labelledby={`${idTabPrefix}-export`}
							className="dialog-tabpanel"
						>
							<p className="export-description">
								Export your project data including all nodes, links, and
								metadata as a JSON file.
							</p>

							{exportData && (
								<div className="export-info">
									<h4 className="export-info-title">Export Details:</h4>
									<p className="export-info-item">
										Nodes: {exportData.nodes?.length ?? 0}
									</p>
									<p className="export-info-item">
										Links: {exportData.nodelinks?.length ?? 0}
									</p>
									<p className="export-info-item">
										Size:{" "}
										{(JSON.stringify(exportData).length / 1024).toFixed(2)} KB
									</p>
								</div>
							)}

							<div className="export-actions">
								<button
									type="button"
									onClick={handleExport}
									disabled={loading}
									className="btn btn-primary"
								>
									{loading ? "Exporting..." : "Export as JSON"}
								</button>

								<button
									type="button"
									onClick={handleExportMermaid}
									disabled={loading}
									className="btn btn-primary"
								>
									{loading ? "Exporting..." : "Export as Mermaid"}
								</button>

								<button
									type="button"
									onClick={handleViewMermaid}
									disabled={loading}
									className="btn btn-primary"
								>
									{loading ? "Loading..." : "View as Mermaid"}
								</button>
							</div>
						</div>
					)}

					{/* Import Tab */}
					{activeTab === "import" && (
						<div
							role="tabpanel"
							id={`${idTabPanelPrefix}-import`}
							aria-labelledby={`${idTabPrefix}-import`}
							className="dialog-tabpanel"
						>
							<p className="export-description">
								Import a project from a JSON file.
							</p>

							<input
								id={idImportFile}
								type="file"
								accept=".json,application/json"
								onChange={handleImportFileChange}
								className="import-file-input"
							/>
							<label htmlFor={idImportFile} className="btn btn-primary">
								Choose Import File
							</label>

							<label
								htmlFor={idImportFile}
								className={`import-dropzone ${importDragActive ? "import-dropzone-active" : ""}`}
								onDrop={handleImportDrop}
								onDragOver={(event) => {
									event.preventDefault();
									setImportDragActive(true);
								}}
								onDragLeave={(event) => {
									event.preventDefault();
									setImportDragActive(false);
								}}
							>
								<p className="import-dropzone-title">
									Drop project export JSON here
								</p>
								<p>or use the file selector above</p>
							</label>

							<div className="form-group">
								<label className="form-label" htmlFor={idImportMode}>
									Import Mode
								</label>
								<select
									id={idImportMode}
									value={importMode}
									onChange={(event) => {
										setImportMode(event.target.value as ImportMode);
										setOverwriteConfirmed(false);
									}}
									className="form-input"
								>
									<option value="new">Create New Project</option>
									<option value="merge">Merge Into Current Project</option>
									<option value="overwrite">Overwrite Current Project</option>
								</select>
							</div>

							{importMode === "overwrite" && (
								<div className="delete-warning">
									<p className="delete-warning-title">
										⚠️ Overwrite will remove current nodes and links
									</p>
									<label
										className="import-overwrite-confirm"
										htmlFor={idOverwriteConfirm}
									>
										<input
											id={idOverwriteConfirm}
											type="checkbox"
											checked={overwriteConfirmed}
											onChange={(event) =>
												setOverwriteConfirmed(event.target.checked)
											}
										/>
										<span>
											I understand overwrite cannot be undone and will replace
											this project data.
										</span>
									</label>
								</div>
							)}

							{importData && (
								<div className="export-info">
									<h4 className="export-info-title">Import Preview:</h4>
									<p className="export-info-item">File: {importFileName}</p>
									<p className="export-info-item">
										Project: {importData.project.name}
									</p>
									<p className="export-info-item">
										Nodes: {importData.nodes.length}
									</p>
									<p className="export-info-item">
										Links: {importData.nodelinks.length}
									</p>
									<p className="export-info-item">
										Attachments: {importData.attachments.length}
									</p>
								</div>
							)}

							<div className="export-actions">
								<button
									type="button"
									onClick={handleImport}
									disabled={
										loading ||
										!importData ||
										(importMode === "overwrite" && !overwriteConfirmed)
									}
									className="btn btn-primary"
								>
									{loading ? "Importing..." : "Import Project"}
								</button>
							</div>
						</div>
					)}

					{/* Delete Tab */}
					{activeTab === "delete" && (
						<div
							role="tabpanel"
							id={`${idTabPanelPrefix}-delete`}
							aria-labelledby={`${idTabPrefix}-delete`}
							className="dialog-tabpanel"
						>
							{currentProject.id === "00000000-0000-0000-0000-000000000000" ? (
								<div className="delete-warning">
									<p className="delete-warning-title">
										🔒 Inbox Project Cannot Be Deleted
									</p>
									<p className="delete-warning-text">
										The Inbox project is a default system project and cannot be
										deleted. It serves as a fallback for organizing your initial
										work.
									</p>
								</div>
							) : (
								<>
									<div className="delete-warning">
										<p className="delete-warning-title">
											⚠️ Warning: This action cannot be undone
										</p>
										<p className="delete-warning-text">
											Deleting this project will permanently remove all nodes,
											links, and associated data.
										</p>
									</div>

									<div className="form-group">
										<label className="form-label" htmlFor={idDeleteConfirmName}>
											Type project name to confirm:{" "}
											<strong>{currentProject.name}</strong>
										</label>
										<input
											type="text"
											id={idDeleteConfirmName}
											value={deleteConfirmName}
											onChange={(e) => setDeleteConfirmName(e.target.value)}
											className="form-input"
											placeholder="Enter project name"
										/>
									</div>

									<button
										type="button"
										onClick={handleDelete}
										disabled={
											loading || deleteConfirmName !== currentProject.name
										}
										className="btn btn-danger"
									>
										{loading ? "Deleting..." : "Delete Project"}
									</button>
								</>
							)}
						</div>
					)}
				</div>
			</div>

			{/* Mermaid Viewer Dialog */}
			<MermaidViewerDialog
				isOpen={mermaidViewerOpen}
				onClose={() => {
					setMermaidViewerOpen(false);
					// Update URL to remove mermaid view parameter
					window.location.hash = `project=${currentProject.id}`;
				}}
				mermaidCode={mermaidCode}
				projectName={currentProject.name}
			/>
		</div>
	);
};
