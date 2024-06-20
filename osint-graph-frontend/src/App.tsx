import { useEffect, useMemo, useState } from "react";
import { DirectedGraph } from "graphology";
import {
	FullScreenControl,
	SigmaContainer,
	ZoomControl,
} from "@react-sigma/core";
import { createNodeImageProgram } from "@sigma/node-image";

import { BiBookContent, BiRadioCircleMarked } from "react-icons/bi";
import {
	BsArrowsFullscreen,
	BsFullscreenExit,
	BsZoomIn,
	BsZoomOut,
} from "react-icons/bs";
import { GrClose } from "react-icons/gr";

import { omit } from "lodash";
import type { Settings } from "sigma/settings";

// import type { MouseEvent as ReactMouseEvent } from "react";

// import { type CustomNode, initialNodes, nodeTypes } from "./nodes";
// import { type CustomEdge, initialEdges, edgeTypes } from "./edges";
// import { newNode } from "./ui";
// import { fetchProjects } from "./api";
import type { Dataset, FiltersState } from "./types";
import { drawHover, drawLabel } from "./sigmademo/canvas-utils";
import GraphSettingsController from "./sigmademo/GraphSettingsController";
import GraphEventsController from "./sigmademo/GraphEventsController";
import GraphDataController from "./sigmademo/GraphDataController";
import GraphTitle from "./sigmademo/GraphTitle";
import SearchField from "./sigmademo/SearchField";
import DescriptionPanel from "./sigmademo/DescriptionPanel";
import ClustersPanel from "./sigmademo/ClustersPanel";
import TagsPanel from "./sigmademo/TagsPanel";

// /** pulls the project list  */
// function updateProjects(
// 	setProjectNodes: React.Dispatch<React.SetStateAction<Project[]>>,
// ) {
// 	fetchProjects().then((projects) => {
// 		console.debug("projects loaded", projects);
// 		setProjectNodes(projects);
// 	});
// }

const baseDataSet = {
	nodes: [
		{
			key: "cytoscape",
			label: "Cytoscape",
			tag: "Tool",
			URL: "https://en.wikipedia.org/wiki/Cytoscape",
			cluster: "1",
			x: 643.82275390625,
			y: -770.3126220703125,
			score: 0.00006909602204225056,
		},
		{
			key: "microsoft excel",
			label: "Microsoft Excel",
			tag: "Tool",
			URL: "https://en.wikipedia.org/wiki/Microsoft%20Excel",
			cluster: "1",
			x: -857.2847900390625,
			y: 602.7734375,
			score: 0.0018317394731443256,
		},
	],
	edges: [["cytoscape", "microsoft excel"]] as [string, string][],
	clusters: [{ key: "1", color: "#6c3e81", clusterLabel: "Graph theory" }],
	tags: [
		{ key: "Chart type", image: "charttype.svg" },
		{ key: "Company", image: "company.svg" },
		{ key: "Concept", image: "concept.svg" },
		{ key: "Field", image: "field.svg" },
		{ key: "List", image: "list.svg" },
		{ key: "Method", image: "method.svg" },
		{ key: "Organization", image: "organization.svg" },
		{ key: "Person", image: "person.svg" },
		{ key: "Technology", image: "technology.svg" },
		{ key: "Tool", image: "tool.svg" },
		{ key: "unknown", image: "unknown.svg" },
	],
};

// const emptyDataset: Dataset = {
// 	nodes: [],
// 	edges: [],
// 	clusters: [],
// 	tags: [],
// };

export default function App() {
	const [dataset, _setDataset] = useState<Dataset>(baseDataSet);
	const [dataReady, setDataReady] = useState(false);
	const [showContents, setShowContents] = useState(false);

	const [filtersState, setFiltersState] = useState<FiltersState>({
		clusters: {},
		tags: {},
	});

	// on startup, pull the project list
	useEffect(() => {
		// updateProjects(setProjectNodes);
		setDataReady(true);
	}, []); // Empty array means this effect runs once on component mount

	const [hoveredNode, setHoveredNode] = useState<string | null>(null);
	const sigmaSettings: Partial<Settings> = useMemo(
		() => ({
			nodeProgramClasses: {
				image: createNodeImageProgram({
					size: { mode: "force", value: 256 },
				}),
			},
			defaultDrawNodeLabel: drawLabel,
			defaultDrawNodeHover: drawHover,
			defaultNodeType: "image",
			defaultEdgeType: "arrow",
			labelDensity: 0.07,
			labelGridCellSize: 60,
			labelRenderedSizeThreshold: 15,
			labelFont: "sans-serif",
			zIndex: true,
			allowInvalidContainer: true, // TODO: work out why this is being derpy
		}),
		[],
	);

	// if (!dataReady) {
	// 	console.debug("No data!");
	// 	return null;
	// }

	return (
		<div id="app-root" className={showContents ? "show-contents" : ""}>
			<SigmaContainer
				graph={DirectedGraph}
				settings={sigmaSettings}
				className="react-sigma"
			>
				<GraphSettingsController hoveredNode={hoveredNode} />
				<GraphEventsController setHoveredNode={setHoveredNode} />
				<GraphDataController dataset={dataset} filters={filtersState} />

				{dataReady && (
					<>
						<div className="controls">
							<div className="react-sigma-control ico">
								<button
									type="button"
									className="show-contents"
									onClick={() => setShowContents(true)}
									title="Show caption and description"
								>
									<BiBookContent />
								</button>
							</div>
							<FullScreenControl className="ico">
								<BsArrowsFullscreen />
								<BsFullscreenExit />
							</FullScreenControl>

							<ZoomControl className="ico">
								<BsZoomIn />
								<BsZoomOut />
								<BiRadioCircleMarked />
							</ZoomControl>
						</div>
						<div className="contents">
							<div className="ico">
								<button
									type="button"
									className="ico hide-contents"
									onClick={() => setShowContents(false)}
									title="Show caption and description"
								>
									<GrClose />
								</button>
							</div>
							<GraphTitle filters={filtersState} />
							<div className="panels">
								<SearchField filters={filtersState} />
								<DescriptionPanel />
								<ClustersPanel
									clusters={dataset.clusters}
									filters={filtersState}
									setClusters={(clusters) =>
										setFiltersState((filters) => ({
											...filters,
											clusters,
										}))
									}
									toggleCluster={(cluster) => {
										setFiltersState((filters) => ({
											...filters,
											clusters: filters.clusters[cluster]
												? omit(filters.clusters, cluster)
												: { ...filters.clusters, [cluster]: true },
										}));
									}}
								/>
								<TagsPanel
									tags={dataset.tags}
									filters={filtersState}
									setTags={(tags) =>
										setFiltersState((filters) => ({
											...filters,
											tags,
										}))
									}
									toggleTag={(tag) => {
										setFiltersState((filters) => ({
											...filters,
											tags: filters.tags[tag]
												? omit(filters.tags, tag)
												: { ...filters.tags, [tag]: true },
										}));
									}}
								/>
							</div>
						</div>
					</>
				)}
			</SigmaContainer>
		</div>
	);
}
