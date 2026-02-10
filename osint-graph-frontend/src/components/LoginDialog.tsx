import { useId } from "react";
import { useAuth } from "../contexts/AuthContext";

export function LoginDialog() {
	const {
		showLoginDialog,
		handleLogin,
		dismissLogin,
		pendingRequests,
		clearQueue,
		isRetrying,
		retryStatus,
	} = useAuth();
	const titleId = useId();

	if (!showLoginDialog) {
		return null;
	}

	return (
		<div
			role="dialog"
			aria-modal="true"
			aria-labelledby={titleId}
			className="login-dialog-backdrop"
			onClick={(e) => {
				// Close on backdrop click
				if (e.target === e.currentTarget && !isRetrying) {
					dismissLogin();
				}
			}}
			onKeyDown={(e) => {
				if (e.key === "Escape" && !isRetrying) {
					dismissLogin();
				}
			}}
		>
			<div className="login-dialog-content">
				<div id={titleId} className="login-dialog-title">
					{isRetrying
						? "🔄 Retrying Requests..."
						: "🔒 Authentication Required"}
				</div>

				{/* Show what failed */}
				{pendingRequests.length > 0 && (
					<div className="login-dialog-context">
						<p>
							The following action{pendingRequests.length > 1 ? "s" : ""} failed
							due to session expiry:
						</p>
						<ul className="failed-actions-list">
							{pendingRequests.map((req) => (
								<li key={req.id} className="failed-action-item">
									<span className="action-icon">⚠️</span>
									<span className="action-text">{req.userAction}</span>
									{retryStatus.get(req.id) && (
										<span
											className={`retry-status retry-status-${retryStatus.get(req.id)}`}
										>
											{retryStatus.get(req.id)}
										</span>
									)}
								</li>
							))}
						</ul>
						<p className="retry-info">
							After you log in, we'll automatically retry{" "}
							{pendingRequests.length > 1 ? "these actions" : "this action"}.
						</p>
					</div>
				)}

				{/* Generic message for no queued requests */}
				{pendingRequests.length === 0 && (
					<div className="login-dialog-message">
						Your session has expired. Please log in to continue.
					</div>
				)}

				{/* Retry in progress indicator */}
				{isRetrying && (
					<div className="retry-progress">
						<div className="retry-spinner" />
						<span>Retrying your requests...</span>
					</div>
				)}

				<div className="login-dialog-buttons">
					<button
						type="button"
						onClick={handleLogin}
						className="btn btn-primary login-dialog-button"
						disabled={isRetrying}
					>
						{isRetrying ? "Retrying..." : "Log In"}
					</button>

					{pendingRequests.length > 0 && !isRetrying && (
						<button
							type="button"
							onClick={() => {
								clearQueue();
								dismissLogin();
							}}
							className="btn btn-danger login-dialog-button"
						>
							Cancel & Clear Queue
						</button>
					)}

					<button
						type="button"
						onClick={dismissLogin}
						className="btn btn-secondary login-dialog-button"
						disabled={isRetrying}
					>
						Dismiss
					</button>
				</div>
			</div>
		</div>
	);
}
