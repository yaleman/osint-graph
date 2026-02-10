import type React from "react";
import { useCallback, useEffect, useRef, useState } from "react";
import "../osint-graph.css";

interface MermaidViewerDialogProps {
	isOpen: boolean;
	onClose: () => void;
	mermaidCode: string;
	projectName: string;
}

export const MermaidViewerDialog: React.FC<MermaidViewerDialogProps> = ({
	isOpen,
	onClose,
	mermaidCode,
	projectName,
}) => {
	const containerRef = useRef<HTMLDivElement>(null);
	const [mermaidLoaded, setMermaidLoaded] = useState(false);
	const mermaidRef = useRef<typeof import("mermaid").default | null>(null);
	const [zoom, setZoom] = useState(1);
	const [pan, setPan] = useState({ x: 0, y: 0 });
	const [isDragging, setIsDragging] = useState(false);
	const [dragStart, setDragStart] = useState({ x: 0, y: 0 });

	const renderDiagram = useCallback(async () => {
		if (!containerRef.current || !mermaidRef.current) return;

		try {
			// Clear previous content by removing all children
			while (containerRef.current.firstChild) {
				containerRef.current.removeChild(containerRef.current.firstChild);
			}

			// Create a unique ID for this diagram
			const id = `mermaid-${Date.now()}`;

			// Render the diagram
			const { svg } = await mermaidRef.current.render(id, mermaidCode);

			// Create a temporary container to parse the SVG string
			const tempDiv = document.createElement("div");
			const range = document.createRange();
			const fragment = range.createContextualFragment(svg);
			tempDiv.appendChild(fragment);
			const svgElement = tempDiv.querySelector("svg");

			if (svgElement && containerRef.current) {
				svgElement.classList.add("mermaid-rendered-svg");
				containerRef.current.appendChild(svgElement);
			}
		} catch (error) {
			console.error("Error rendering Mermaid diagram:", error);
			if (containerRef.current) {
				// Clear container by removing all children
				while (containerRef.current.firstChild) {
					containerRef.current.removeChild(containerRef.current.firstChild);
				}

				// Create error container
				const errorDiv = document.createElement("div");
				errorDiv.className = "mermaid-render-error";

				// Create error title
				const errorTitle = document.createElement("h3");
				errorTitle.className = "mermaid-render-error-title";
				errorTitle.textContent = "Error rendering diagram";
				errorDiv.appendChild(errorTitle);

				// Create error message
				const errorPre = document.createElement("pre");
				errorPre.className = "mermaid-render-error-body";
				errorPre.textContent = String(error);
				errorDiv.appendChild(errorPre);

				// Create code title
				const codeTitle = document.createElement("h4");
				codeTitle.className = "mermaid-render-error-title";
				codeTitle.textContent = "Mermaid Code:";
				errorDiv.appendChild(codeTitle);

				// Create code display
				const codePre = document.createElement("pre");
				codePre.className = "mermaid-render-error-body";
				codePre.textContent = mermaidCode;
				errorDiv.appendChild(codePre);

				containerRef.current.appendChild(errorDiv);
			}
		}
	}, [mermaidCode]);

	const handleZoomIn = () => {
		setZoom((prev) => Math.min(prev + 0.3, 100));
	};

	const handleZoomOut = () => {
		setZoom((prev) => Math.max(prev - 0.3, 0.1));
	};

	const handleResetZoom = () => {
		setZoom(1);
		setPan({ x: 0, y: 0 });
	};

	const handleMouseDown = (e: React.MouseEvent) => {
		if (e.button === 0) {
			// Left click
			setIsDragging(true);
			setDragStart({ x: e.clientX - pan.x, y: e.clientY - pan.y });
		}
	};

	const handleMouseMove = (e: React.MouseEvent) => {
		if (isDragging) {
			setPan({
				x: e.clientX - dragStart.x,
				y: e.clientY - dragStart.y,
			});
		}
	};

	const handleMouseUp = () => {
		setIsDragging(false);
	};

	const handleWheel = (e: React.WheelEvent) => {
		e.preventDefault();
		const delta = e.deltaY > 0 ? -0.15 : 0.15;
		setZoom((prev) => Math.max(0.1, Math.min(10, prev + delta)));
	};

	useEffect(() => {
		// Dynamically import mermaid when the component is first used
		if (!mermaidRef.current) {
			import("mermaid")
				.then((m) => {
					mermaidRef.current = m.default;
					m.default.initialize({ startOnLoad: false });
					setMermaidLoaded(true);
				})
				.catch((error) => {
					console.error("Failed to load Mermaid:", error);
				});
		}
	}, []);

	useEffect(() => {
		if (isOpen && mermaidLoaded) {
			renderDiagram();
		}
	}, [isOpen, mermaidLoaded, renderDiagram]);

	useEffect(() => {
		if (!containerRef.current) return;
		containerRef.current.style.transform = `translate(${pan.x}px, ${pan.y}px) scale(${zoom})`;
		containerRef.current.style.transition = isDragging
			? "none"
			: "transform 0.1s ease-out";
	}, [pan, zoom, isDragging]);

	// Hide the node panel when the dialog is open
	useEffect(() => {
		if (isOpen) {
			document.body.classList.add("mermaid-viewer-active");
		} else {
			document.body.classList.remove("mermaid-viewer-active");
		}
		return () => {
			document.body.classList.remove("mermaid-viewer-active");
		};
	}, [isOpen]);

	if (!isOpen) return null;

	return (
		<div
			role="dialog"
			className="dialog-backdrop mermaid-fullscreen-backdrop"
			onClick={onClose}
			onKeyDown={() => {}}
		>
			<div
				role="dialog"
				className="dialog-container mermaid-fullscreen-container"
				onClick={(e) => e.stopPropagation()}
				onKeyDown={() => {}}
			>
				{/* Header */}
				<div className="dialog-header">
					<h2 className="dialog-title">Mermaid Diagram: {projectName}</h2>
					<div className="mermaid-header-controls">
						{/* Zoom Controls */}
						<div className="mermaid-zoom-controls">
							<button
								type="button"
								onClick={handleZoomOut}
								className="btn btn-primary"
								title="Zoom Out"
							>
								−
							</button>
							<span className="zoom-level">{Math.round(zoom * 100)}%</span>
							<button
								type="button"
								onClick={handleZoomIn}
								className="btn btn-primary"
								title="Zoom In"
							>
								+
							</button>
							<button
								type="button"
								onClick={handleResetZoom}
								className="btn btn-primary"
								title="Reset Zoom"
							>
								Reset
							</button>
						</div>
						<button
							type="button"
							onClick={onClose}
							className="btn btn-transparent"
						>
							×
						</button>
					</div>
				</div>

				{/* Content */}
				<div
					role="dialog"
					className={`dialog-content mermaid-content mermaid-pan-surface ${
						isDragging ? "mermaid-pan-surface-dragging" : ""
					}`}
					onMouseDown={handleMouseDown}
					onMouseMove={handleMouseMove}
					onMouseUp={handleMouseUp}
					onMouseLeave={handleMouseUp}
					onWheel={handleWheel}
					onKeyDown={() => {}}
				>
					<div
						ref={containerRef}
						className="mermaid-container mermaid-pan-container"
					/>
				</div>
			</div>
		</div>
	);
};
