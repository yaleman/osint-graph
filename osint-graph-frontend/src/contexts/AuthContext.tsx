import type React from "react";
import { createContext, useCallback, useContext, useState } from "react";

interface AuthContextType {
	showLoginDialog: boolean;
	requireLogin: () => void;
	dismissLogin: () => void;
	handleLogin: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
	const [showLoginDialog, setShowLoginDialog] = useState(false);

	const requireLogin = useCallback(() => {
		setShowLoginDialog(true);
	}, []);

	const dismissLogin = useCallback(() => {
		setShowLoginDialog(false);
	}, []);

	const handleLogin = useCallback(() => {
		// Redirect to the backend login endpoint which will start OAuth flow
		window.location.href = "/admin/login";
	}, []);

	return (
		<AuthContext.Provider
			value={{
				showLoginDialog,
				requireLogin,
				dismissLogin,
				handleLogin,
			}}
		>
			{children}
		</AuthContext.Provider>
	);
}

export function useAuth() {
	const context = useContext(AuthContext);
	if (context === undefined) {
		throw new Error("useAuth must be used within an AuthProvider");
	}
	return context;
}
