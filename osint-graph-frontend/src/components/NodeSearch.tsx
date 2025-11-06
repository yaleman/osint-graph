import { useCallback, useEffect, useRef, useState } from "react";
import type { Node } from "reactflow";
import { getNodeColor } from "../types";

interface NodeSearchProps {
	nodes: Node[];
	onNodeSelect: (nodeId: string) => void;
}

export function NodeSearch({ nodes, onNodeSelect }: NodeSearchProps) {
	const [searchTerm, setSearchTerm] = useState("");
	const [searchResults, setSearchResults] = useState<Node[]>([]);
	const [isOpen, setIsOpen] = useState(false);
	const searchRef = useRef<HTMLDivElement>(null);

	// Search through nodes
	const performSearch = useCallback(
		(term: string) => {
			if (!term.trim()) {
				setSearchResults([]);
				setIsOpen(false);
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

			setSearchResults(results);
			setIsOpen(results.length > 0);
		},
		[nodes],
	);

	// Handle search input change
	const handleSearchChange = (e: React.ChangeEvent<HTMLInputElement>) => {
		const term = e.target.value;
		setSearchTerm(term);
		performSearch(term);
	};

	// Handle clicking on a search result
	const handleResultClick = (nodeId: string) => {
		onNodeSelect(nodeId);
		setSearchTerm("");
		setSearchResults([]);
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

	return (
		<div className="node-search-container" ref={searchRef}>
			<input
				type="text"
				className="node-search-input"
				placeholder="Search nodes..."
				value={searchTerm}
				onChange={handleSearchChange}
				onFocus={() => {
					if (searchResults.length > 0) {
						setIsOpen(true);
					}
				}}
			/>
			{isOpen && searchResults.length > 0 && (
				<div className="node-search-results">
					{searchResults.map((node) => {
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
								onClick={() => handleResultClick(node.id)}
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
				</div>
			)}
		</div>
	);
}
