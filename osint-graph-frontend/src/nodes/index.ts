import type { Node, NodeTypes } from "reactflow";
import { PositionLoggerNode } from "./PositionLoggerNode";
import {
  PersonNode,
  ImageNode,
  DomainNode,
  IPNode,
  PhoneNode,
  URLNode,
  EmailNode,
  LocationNode,
  OrganizationNode,
  DocumentNode,
} from "./OSINTNodes";
import type { SimulationNodeDatum } from "d3-force";

export interface CustomNode extends Node, SimulationNodeDatum {
  id: string;
  vx?: number;
  vy?: number;

}




export const initialNodes = [
  { id: "a", type: "input", position: { x: 0, y: 0 }, vx: 0.0, vy: 0.0, data: { label: "wire" } },
  {
    id: "b",
    type: "position-logger",
    position: { x: -100, y: 100 },
    data: { label: "drag me!" },
  },
  { id: "c", position: { x: 100, y: 100 }, data: { label: "your head" } },
  {
    id: "d",
    type: "output",
    position: { x: 0, y: 200 },
    data: { label: "with React Flow" },
  },
] satisfies CustomNode[];

export const nodeTypes = {
  "position-logger": PositionLoggerNode,
  "person": PersonNode,
  "image": ImageNode,
  "domain": DomainNode,
  "ip": IPNode,
  "phone": PhoneNode,
  "url": URLNode,
  "email": EmailNode,
  "location": LocationNode,
  "organization": OrganizationNode,
  "document": DocumentNode,
} satisfies NodeTypes;
