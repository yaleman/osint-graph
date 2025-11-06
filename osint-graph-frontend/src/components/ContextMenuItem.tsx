import type { Node } from "reactflow";

interface ContextMenuItemProps {
	node: Node;
	onClick: () => void;
	applicableNodeTypes: string[];
	icon: string;
	title: string;
}

export function ContextMenuItem({
	node,
	onClick,
	applicableNodeTypes,
	icon,
	title,
}: ContextMenuItemProps) {
	// eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
	const nodeType = node.data.nodeType as string;

	// Only render if this node type is in the applicable list
	if (!applicableNodeTypes.includes(nodeType)) {
		return null;
	}

	return (
		<button type="button" onClick={onClick} className="context-menu-item">
			<span>{icon}</span>
			<span>{title}</span>
		</button>
	);
}
