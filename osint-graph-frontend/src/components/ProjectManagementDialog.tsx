import React, { useState, useEffect } from "react";
import toast from "react-hot-toast";
import { updateProject, deleteProject, exportProject } from "../api";
import type { Project, ProjectExport } from "../types";

interface ProjectManagementDialogProps {
	isOpen: boolean;
	onClose: () => void;
	currentProject: Project | null;
	onProjectUpdate: (project: Project) => void;
	onProjectDelete: () => void;
}

type TabType = "general" | "export" | "import" | "delete";

export const ProjectManagementDialog: React.FC<ProjectManagementDialogProps> = ({
	isOpen,
	onClose,
	currentProject,
	onProjectUpdate,
	onProjectDelete,
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
				name: projectName,
				description: projectDescription || undefined,
				tags: projectTags,
			});
			onProjectUpdate(updatedProject);
			toast.success("Project updated successfully");
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
			const data = await exportProject(currentProject.id);
			setExportData(data);

			// Create download
			const blob = new Blob([JSON.stringify(data, null, 2)], {
				type: "application/json",
			});
			const url = URL.createObjectURL(blob);
			const a = document.createElement("a");
			a.href = url;
			const timestamp = new Date().toISOString().replace(/[:.]/g, "-").split("T")[0];
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

		if (!window.confirm("Are you sure you want to delete this project? This action cannot be undone.")) {
			return;
		}

		setLoading(true);
		try {
			await deleteProject(currentProject.id);
			toast.success("Project deleted successfully");
			onProjectDelete();
			onClose();
		} catch (error) {
			console.error("Failed to delete project:", error);
			toast.error("Failed to delete project");
		} finally {
			setLoading(false);
		}
	};

	const tabStyle = (tab: TabType): React.CSSProperties => ({
		padding: "10px 20px",
		cursor: "pointer",
		borderBottom: activeTab === tab ? "2px solid #3b82f6" : "2px solid transparent",
		color: activeTab === tab ? "#3b82f6" : "#6b7280",
		fontWeight: activeTab === tab ? "600" : "400",
		transition: "all 0.2s",
	});

	return (
		<div
			style={{
				position: "fixed",
				top: 0,
				left: 0,
				right: 0,
				bottom: 0,
				backgroundColor: "rgba(0, 0, 0, 0.5)",
				display: "flex",
				alignItems: "center",
				justifyContent: "center",
				zIndex: 1000,
			}}
			onClick={onClose}
		>
			<div
				style={{
					backgroundColor: "white",
					borderRadius: "8px",
					width: "600px",
					maxWidth: "90vw",
					maxHeight: "80vh",
					overflow: "hidden",
					display: "flex",
					flexDirection: "column",
					boxShadow: "0 4px 6px rgba(0, 0, 0, 0.1)",
				}}
				onClick={(e) => e.stopPropagation()}
			>
				{/* Header */}
				<div
					style={{
						padding: "20px",
						borderBottom: "1px solid #e5e7eb",
						display: "flex",
						justifyContent: "space-between",
						alignItems: "center",
					}}
				>
					<h2 style={{ margin: 0, fontSize: "20px", fontWeight: "600" }}>Project Settings</h2>
					<button
						onClick={onClose}
						style={{
							background: "none",
							border: "none",
							fontSize: "24px",
							cursor: "pointer",
							color: "#6b7280",
							padding: "0",
							width: "30px",
							height: "30px",
						}}
					>
						×
					</button>
				</div>

				{/* Tabs */}
				<div
					style={{
						display: "flex",
						borderBottom: "1px solid #e5e7eb",
						backgroundColor: "#f9fafb",
					}}
				>
					<div style={tabStyle("general")} onClick={() => setActiveTab("general")}>
						General
					</div>
					<div style={tabStyle("export")} onClick={() => setActiveTab("export")}>
						Export
					</div>
					<div style={tabStyle("import")} onClick={() => setActiveTab("import")}>
						Import
					</div>
					<div style={tabStyle("delete")} onClick={() => setActiveTab("delete")}>
						Delete
					</div>
				</div>

				{/* Tab Content */}
				<div
					style={{
						padding: "20px",
						overflowY: "auto",
						flex: 1,
					}}
				>
					{/* General Tab */}
					{activeTab === "general" && (
						<div>
							<div style={{ marginBottom: "20px" }}>
								<label
									style={{
										display: "block",
										marginBottom: "5px",
										fontWeight: "500",
										color: "#374151",
									}}
								>
									Project Name *
								</label>
								<input
									type="text"
									value={projectName}
									onChange={(e) => setProjectName(e.target.value)}
									style={{
										width: "100%",
										padding: "8px 12px",
										border: "1px solid #d1d5db",
										borderRadius: "4px",
										fontSize: "14px",
									}}
									placeholder="Enter project name"
								/>
							</div>

							<div style={{ marginBottom: "20px" }}>
								<label
									style={{
										display: "block",
										marginBottom: "5px",
										fontWeight: "500",
										color: "#374151",
									}}
								>
									Description
								</label>
								<textarea
									value={projectDescription}
									onChange={(e) => setProjectDescription(e.target.value)}
									style={{
										width: "100%",
										padding: "8px 12px",
										border: "1px solid #d1d5db",
										borderRadius: "4px",
										fontSize: "14px",
										minHeight: "100px",
										resize: "vertical",
									}}
									placeholder="Enter project description"
								/>
							</div>

							<div style={{ marginBottom: "20px" }}>
								<label
									style={{
										display: "block",
										marginBottom: "5px",
										fontWeight: "500",
										color: "#374151",
									}}
								>
									Tags
								</label>
								<div style={{ display: "flex", gap: "8px", marginBottom: "10px", flexWrap: "wrap" }}>
									{projectTags.map((tag) => (
										<div
											key={tag}
											style={{
												backgroundColor: "#3b82f6",
												color: "white",
												padding: "4px 12px",
												borderRadius: "16px",
												fontSize: "14px",
												display: "flex",
												alignItems: "center",
												gap: "8px",
											}}
										>
											{tag}
											<button
												onClick={() => handleRemoveTag(tag)}
												style={{
													background: "none",
													border: "none",
													color: "white",
													cursor: "pointer",
													fontSize: "16px",
													padding: "0",
													lineHeight: "1",
												}}
											>
												×
											</button>
										</div>
									))}
								</div>
								<div style={{ display: "flex", gap: "8px" }}>
									<input
										type="text"
										value={newTag}
										onChange={(e) => setNewTag(e.target.value)}
										onKeyPress={(e) => {
											if (e.key === "Enter") {
												e.preventDefault();
												handleAddTag();
											}
										}}
										style={{
											flex: 1,
											padding: "8px 12px",
											border: "1px solid #d1d5db",
											borderRadius: "4px",
											fontSize: "14px",
										}}
										placeholder="Add a tag"
									/>
									<button
										onClick={handleAddTag}
										style={{
											padding: "8px 16px",
											backgroundColor: "#3b82f6",
											color: "white",
											border: "none",
											borderRadius: "4px",
											cursor: "pointer",
											fontSize: "14px",
											fontWeight: "500",
										}}
									>
										Add
									</button>
								</div>
							</div>

							<button
								onClick={handleSaveGeneral}
								disabled={loading || !projectName.trim()}
								style={{
									padding: "10px 20px",
									backgroundColor: loading || !projectName.trim() ? "#9ca3af" : "#3b82f6",
									color: "white",
									border: "none",
									borderRadius: "4px",
									cursor: loading || !projectName.trim() ? "not-allowed" : "pointer",
									fontSize: "14px",
									fontWeight: "500",
									width: "100%",
								}}
							>
								{loading ? "Saving..." : "Save Changes"}
							</button>
						</div>
					)}

					{/* Export Tab */}
					{activeTab === "export" && (
						<div>
							<p style={{ color: "#6b7280", marginBottom: "20px" }}>
								Export your project data including all nodes, links, and metadata as a JSON file.
							</p>

							{exportData && (
								<div
									style={{
										backgroundColor: "#f3f4f6",
										padding: "15px",
										borderRadius: "4px",
										marginBottom: "20px",
									}}
								>
									<h4 style={{ margin: "0 0 10px 0", fontSize: "14px", fontWeight: "600" }}>
										Export Details:
									</h4>
									<p style={{ margin: "5px 0", fontSize: "14px", color: "#6b7280" }}>
										Nodes: {exportData.nodes?.length ?? 0}
									</p>
									<p style={{ margin: "5px 0", fontSize: "14px", color: "#6b7280" }}>
										Links: {exportData.links?.length ?? 0}
									</p>
									<p style={{ margin: "5px 0", fontSize: "14px", color: "#6b7280" }}>
										Size: {(JSON.stringify(exportData).length / 1024).toFixed(2)} KB
									</p>
								</div>
							)}

							<button
								onClick={handleExport}
								disabled={loading}
								style={{
									padding: "10px 20px",
									backgroundColor: loading ? "#9ca3af" : "#3b82f6",
									color: "white",
									border: "none",
									borderRadius: "4px",
									cursor: loading ? "not-allowed" : "pointer",
									fontSize: "14px",
									fontWeight: "500",
									width: "100%",
								}}
							>
								{loading ? "Exporting..." : "Export Project"}
							</button>
						</div>
					)}

					{/* Import Tab */}
					{activeTab === "import" && (
						<div>
							<p style={{ color: "#6b7280", marginBottom: "20px" }}>
								Import a project from a JSON file.
							</p>

							<div
								style={{
									border: "2px dashed #d1d5db",
									borderRadius: "8px",
									padding: "40px",
									textAlign: "center",
									color: "#6b7280",
								}}
							>
								<p style={{ marginBottom: "10px", fontWeight: "500" }}>Coming Soon</p>
								<p style={{ fontSize: "14px" }}>
									Project import functionality will be available in a future update.
								</p>
							</div>
						</div>
					)}

					{/* Delete Tab */}
					{activeTab === "delete" && (
						<div>
							<div
								style={{
									backgroundColor: "#fef2f2",
									border: "1px solid #fecaca",
									borderRadius: "4px",
									padding: "15px",
									marginBottom: "20px",
								}}
							>
								<p style={{ color: "#991b1b", fontWeight: "500", margin: "0 0 10px 0" }}>
									⚠️ Warning: This action cannot be undone
								</p>
								<p style={{ color: "#7f1d1d", fontSize: "14px", margin: 0 }}>
									Deleting this project will permanently remove all nodes, links, and associated data.
								</p>
							</div>

							<div style={{ marginBottom: "20px" }}>
								<label
									style={{
										display: "block",
										marginBottom: "5px",
										fontWeight: "500",
										color: "#374151",
									}}
								>
									Type project name to confirm: <strong>{currentProject.name}</strong>
								</label>
								<input
									type="text"
									value={deleteConfirmName}
									onChange={(e) => setDeleteConfirmName(e.target.value)}
									style={{
										width: "100%",
										padding: "8px 12px",
										border: "1px solid #d1d5db",
										borderRadius: "4px",
										fontSize: "14px",
									}}
									placeholder="Enter project name"
								/>
							</div>

							<button
								onClick={handleDelete}
								disabled={loading || deleteConfirmName !== currentProject.name}
								style={{
									padding: "10px 20px",
									backgroundColor:
										loading || deleteConfirmName !== currentProject.name
											? "#9ca3af"
											: "#dc2626",
									color: "white",
									border: "none",
									borderRadius: "4px",
									cursor:
										loading || deleteConfirmName !== currentProject.name
											? "not-allowed"
											: "pointer",
									fontSize: "14px",
									fontWeight: "500",
									width: "100%",
								}}
							>
								{loading ? "Deleting..." : "Delete Project"}
							</button>
						</div>
					)}
				</div>
			</div>
		</div>
	);
};
