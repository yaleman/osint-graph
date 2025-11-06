import { useId } from "react";
import { useAuth } from "../contexts/AuthContext";

export function LoginDialog() {
	const { showLoginDialog, handleLogin, dismissLogin } = useAuth();
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
				if (e.target === e.currentTarget) {
					dismissLogin();
				}
			}}
			onKeyDown={(e) => {
				if (e.key === "Escape") {
					dismissLogin();
				}
			}}
		>
			<div className="login-dialog-content">
				<div id={titleId} className="login-dialog-title">
					🔒 Authentication Required
				</div>
				<div className="login-dialog-message">
					You need to log in to access this application. Click the button below
					to authenticate.
				</div>
				<div className="login-dialog-buttons">
					<button
						type="button"
						onClick={handleLogin}
						className="btn btn-primary login-dialog-button"
					>
						Log In
					</button>
					<button
						type="button"
						onClick={dismissLogin}
						className="btn btn-secondary login-dialog-button"
					>
						Cancel
					</button>
				</div>
			</div>
		</div>
	);
}
