import { useCallback, useEffect, useRef, useState } from "react";
import type { Node } from "reactflow";
import { searchGlobal } from "../api";
import type { SearchResult } from "../types";
import { getNodeColor } from "../types";

interface NodeSearchProps {
	nodes: Node[];
	onNodeSelect: (nodeId: string) => void;
	onGlobalResultSelect: (nodeId: string, projectId: string) => void;
	currentProjectId: string | null;
	projects: Map<string, string>; // projectId -> projectName
}

export function NodeSearch({
	nodes,
	onNodeSelect,
	onGlobalResultSelect,
	currentProjectId,
	projects,
}: NodeSearchProps) {
	const [searchTerm, setSearchTerm] = useState("");
	const [localResults, setLocalResults] = useState<Node[]>([]);
	const [globalResults, setGlobalResults] = useState<SearchResult[]>([]);
	const [isOpen, setIsOpen] = useState(false);
	const searchRef = useRef<HTMLDivElement>(null);

	// Search through local nodes
	const performLocalSearch = useCallback(
		(term: string) => {
			if (!term.trim()) {
				setLocalResults([]);
				return;
			}

			const lowerTerm = term.toLowerCase();
			const results = nodes.filter((node) => {
				// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
				const osintNode = node.data.osintNode as
					| {
							display?: string;
							value?: string;
							notes?: string;
							node_type?: string;
					  }
					| undefined;

				if (!osintNode) return false;

				// Search in display name
				if (osintNode.display?.toLowerCase().includes(lowerTerm)) return true;
				// Search in value
				if (osintNode.value?.toLowerCase().includes(lowerTerm)) return true;
				// Search in notes
				if (osintNode.notes?.toLowerCase().includes(lowerTerm)) return true;
				// Search in node type
				if (osintNode.node_type?.toLowerCase().includes(lowerTerm)) return true;

				return false;
			});

			setLocalResults(results);
		},
		[nodes],
	);

	// Search globally across all projects
	const performGlobalSearch = useCallback(
		async (term: string) => {
			if (!term.trim()) {
				setGlobalResults([]);
				return;
			}

			try {
				const results = await searchGlobal(term);
				// Filter out results from current project
				const filteredResults = currentProjectId
					? results.filter((r) => r.project_id !== currentProjectId)
					: results;
				setGlobalResults(filteredResults);
			} catch (error) {
				console.error("Global search failed:", error);
				setGlobalResults([]);
			}
		},
		[currentProjectId],
	);

	// Handle search input change
	const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
		const term = e.target.value;
		setSearchTerm(term);
		performLocalSearch(term);
		performGlobalSearch(term);
		setIsOpen(!!term.trim());
	};

	// Handle clicking on a local search result
	const handleLocalResultClick = (nodeId: string) => {
		onNodeSelect(nodeId);
		setSearchTerm("");
		setLocalResults([]);
		setGlobalResults([]);
		setIsOpen(false);
	};

	// Handle clicking on a global search result
	const handleGlobalResultClick = (nodeId: string, projectId: string) => {
		onGlobalResultSelect(nodeId, projectId);
		setSearchTerm("");
		setLocalResults([]);
		setGlobalResults([]);
		setIsOpen(false);
	};

	// Close dropdown when clicking outside
	useEffect(() => {
		const handleClickOutside = (event: MouseEvent) => {
			if (
				searchRef.current &&
				!searchRef.current.contains(event.target as HTMLElement)
			) {
				setIsOpen(false);
			}
		};

		document.addEventListener("mousedown", handleClickOutside);
		return () => document.removeEventListener("mousedown", handleClickOutside);
	}, []);

	// Group global results by project
	const groupedGlobalResults = globalResults.reduce(
		(acc, result) => {
			if (!acc[result.project_id]) {
				acc[result.project_id] = [];
			}
			// biome-ignore lint/style/noNonNullAssertion: We just created it above
			acc[result.project_id]!.push(result);
			return acc;
		},
		{} as Record<string, SearchResult[]>,
	);

	return (
		<div className="node-search-container" ref={searchRef}>
			<input
				type="text"
				className="node-search-input"
				placeholder="Search nodes..."
				value={searchTerm}
				onChange={handleSearchChange}
				onFocus={() => {
					if (localResults.length > 0 || globalResults.length > 0) {
						setIsOpen(true);
					}
				}}
			/>
			{isOpen && (localResults.length > 0 || globalResults.length > 0) && (
				<div className="node-search-results">
					{/* Local results */}
					{localResults.length > 0 && (
						<>
							<div className="node-search-section-header">Current Project</div>
							{localResults.map((node) => {
								// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
								const osintNode = node.data.osintNode as {
									display?: string;
									value?: string;
									node_type?: string;
								};
								const nodeType = osintNode?.node_type || "unknown";
								const nodeColor = getNodeColor(nodeType);
								return (
									<button
										key={node.id}
										type="button"
										className="node-search-result-item"
										onClick={() => handleLocalResultClick(node.id)}
									>
										<div className="node-search-result-title">
											{osintNode?.display || "Unnamed"}
										</div>
										<div className="node-search-result-meta">
											<span
												className="node-search-result-type"
												style={{ backgroundColor: nodeColor, color: "white" }}
											>
												{nodeType}
											</span>
											{osintNode?.value && (
												<span className="node-search-result-value">
													{osintNode.value.length > 50
														? `${osintNode.value.substring(0, 50)}...`
														: osintNode.value}
												</span>
											)}
										</div>
									</button>
								);
							})}
						</>
					)}

					{/* Global results grouped by project */}
					{Object.entries(groupedGlobalResults).map(([projectId, results]) => (
						<div key={projectId}>
							<div className="node-search-section-header">
								{projects.get(projectId) || "Unknown Project"}
							</div>
							{results.map((result) => {
								// Determine node type and color based on result_type
								let nodeType: string | null = null;
								let nodeColor = "#6b7280"; // default gray
								let typeLabel = "";

								if (
									typeof result.result_type === "object" &&
									"Node" in result.result_type
								) {
									// It's a Node result
									nodeType = result.result_type.Node;
									nodeColor = getNodeColor(nodeType);
									typeLabel = nodeType;
								} else if (result.result_type === "Project") {
									typeLabel = "project";
									nodeColor = "#3b82f6"; // blue for projects
								} else if (result.result_type === "Attachment") {
									typeLabel = "attachment";
									nodeColor = "#8b5cf6"; // purple for attachments
								}

								return (
									<button
										key={result.id}
										type="button"
										className="node-search-result-item"
										onClick={() =>
											handleGlobalResultClick(result.id, result.project_id)
										}
									>
										<div className="node-search-result-title">
											{result.title}
										</div>
										<div className="node-search-result-meta">
											{typeLabel && (
												<span
													className="node-search-result-type"
													style={{
														backgroundColor: nodeColor,
														color: "white",
													}}
												>
													{typeLabel}
												</span>
											)}
										</div>
									</button>
								);
							})}
						</div>
					))}
				</div>
			)}
		</div>
	);
}
