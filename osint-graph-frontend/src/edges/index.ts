import type { SimulationLinkDatum } from "d3-force";
import type { Edge, EdgeTypes } from "reactflow";
import type { CustomNode } from "../nodes";

export const initialEdges = [
  { strength: 0.5, source: "a", target: "c", edge: { id: "a->c", source: "a", target: "c", animated: true, }},
  { strength: 0.5, source: "b", target: "d", edge: { id: "b->d", source: "b", target: "d", }},
  { strength: 0.5, source: "c", target: "d", edge: { id: "c->d", source: "c", target: "d", animated: true,  }},
] satisfies CustomEdge[];

export const edgeTypes = {
  // Add your custom edge types here!
} satisfies EdgeTypes;

export type CustomEdge<T = Edge, D=SimulationLinkDatum<CustomNode>> = {
  strength: number;
  source: string | number | CustomNode | undefined ;
  target:  string | number | CustomNode | undefined ;
  edge?: T;
} & D;



// export interface CustomEdge extends SimulationLinkDatum<CustomNode> {
//   strength: number;
//   edge: Edge;
// }