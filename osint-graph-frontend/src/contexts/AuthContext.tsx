import axios, { type AxiosRequestConfig } from "axios";
import type React from "react";
import {
	createContext,
	useCallback,
	useContext,
	useEffect,
	useState,
} from "react";
import { toast } from "react-hot-toast";
import { v4 as uuidv4 } from "uuid";
import type { PendingRequest, RequestStatus } from "../types";

interface AuthContextType {
	showLoginDialog: boolean;
	requireLogin: () => void;
	dismissLogin: () => void;
	handleLogin: () => void;
	pendingRequests: PendingRequest[];
	queueRequest: (config: AxiosRequestConfig, userAction: string) => void;
	clearQueue: () => void;
	retryQueuedRequests: () => Promise<void>;
	isRetrying: boolean;
	retryStatus: Map<string, RequestStatus>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: React.ReactNode }) {
	const [showLoginDialog, setShowLoginDialog] = useState(false);
	const [pendingRequests, setPendingRequests] = useState<PendingRequest[]>([]);
	const [isRetrying, setIsRetrying] = useState(false);
	const [retryStatus, setRetryStatus] = useState<Map<string, RequestStatus>>(
		new Map(),
	);

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

	// Queue a failed request for retry
	const queueRequest = useCallback(
		(config: AxiosRequestConfig, userAction: string) => {
			const request: PendingRequest = {
				id: uuidv4(),
				config,
				timestamp: new Date(),
				userAction,
				attempt: 0,
			};

			setPendingRequests((prev) => {
				// Limit queue size to 10
				if (prev.length >= 10) {
					toast.warning("Request queue full - oldest request discarded");
					return [...prev.slice(1), request];
				}
				return [...prev, request];
			});

			// Update status map
			setRetryStatus((prev) => new Map(prev).set(request.id, "pending"));
		},
		[],
	);

	// Clear the request queue
	const clearQueue = useCallback(() => {
		setPendingRequests([]);
		setRetryStatus(new Map());
	}, []);

	// Retry a single request with exponential backoff
	const retryWithBackoff = useCallback(
		async (request: PendingRequest): Promise<boolean> => {
			const maxAttempts = 3;
			const delays = [1000, 2000, 4000]; // 1s, 2s, 4s

			for (let attempt = 0; attempt < maxAttempts; attempt++) {
				try {
					// Update attempt counter
					request.attempt = attempt + 1;

					// Retry the request
					await axios.request(request.config);
					return true; // Success!
				} catch (error) {
					// If this was the last attempt, give up
					if (attempt === maxAttempts - 1) {
						console.error(
							`Failed to retry request after ${maxAttempts} attempts:`,
							error,
						);
						return false;
					}

					// Wait before next retry with exponential backoff
					await new Promise((resolve) => setTimeout(resolve, delays[attempt]));
				}
			}

			return false;
		},
		[],
	);

	// Retry all queued requests
	const retryQueuedRequests = useCallback(async () => {
		if (pendingRequests.length === 0) return;

		setIsRetrying(true);
		const newStatus = new Map<string, RequestStatus>();

		for (const request of pendingRequests) {
			// Mark as retrying
			newStatus.set(request.id, "retrying");
			setRetryStatus(new Map(newStatus));

			// Attempt retry with backoff
			const success = await retryWithBackoff(request);

			if (success) {
				newStatus.set(request.id, "success");
				toast.success(`Completed: ${request.userAction}`);
			} else {
				newStatus.set(request.id, "failed");
				toast.error(`Failed to retry: ${request.userAction}`);
			}

			setRetryStatus(new Map(newStatus));
		}

		setIsRetrying(false);

		// Remove successfully retried requests from queue
		setPendingRequests((prev) =>
			prev.filter((req) => newStatus.get(req.id) !== "success"),
		);

		// Show summary
		const successful = Array.from(newStatus.values()).filter(
			(s) => s === "success",
		).length;
		const failed = Array.from(newStatus.values()).filter(
			(s) => s === "failed",
		).length;

		if (successful > 0 && failed === 0) {
			toast.success(
				`Successfully retried ${successful} request${successful > 1 ? "s" : ""}`,
			);
		} else if (failed > 0) {
			toast.warning(`Retried ${successful} requests, ${failed} failed`);
		}
	}, [pendingRequests, retryWithBackoff]);

	// Poll for auth restoration when login dialog is open
	useEffect(() => {
		if (!showLoginDialog || pendingRequests.length === 0) return;

		let pollInterval: NodeJS.Timeout;

		const checkAuthStatus = async () => {
			try {
				// Health check - returns 200 if authenticated
				await axios.get("/api/v1/health", {
					validateStatus: (status) => status === 200,
				});

				// Auth restored! Retry queued requests
				await retryQueuedRequests();
				dismissLogin();
			} catch (_error) {
				// Still not authenticated - continue polling
			}
		};

		// Start polling every 2 seconds
		pollInterval = setInterval(checkAuthStatus, 2000);

		// Also check on window focus (handles same-tab return from IDP)
		window.addEventListener("focus", checkAuthStatus);

		return () => {
			clearInterval(pollInterval);
			window.removeEventListener("focus", checkAuthStatus);
		};
	}, [
		showLoginDialog,
		pendingRequests.length,
		retryQueuedRequests,
		dismissLogin,
	]);

	// Timeout warning and clearing
	useEffect(() => {
		if (!showLoginDialog || pendingRequests.length === 0) return;

		// Warning at 5 minutes
		const warningTimeout = setTimeout(
			() => {
				toast.warning(
					"Login dialog has been open for 5 minutes. Queue will be cleared in 5 more minutes.",
				);
			},
			5 * 60 * 1000,
		);

		// Clear at 10 minutes
		const clearTimeout = setTimeout(
			() => {
				clearQueue();
				toast.error("Login timeout - request queue cleared");
			},
			10 * 60 * 1000,
		);

		return () => {
			clearTimeout(warningTimeout);
			clearTimeout(clearTimeout);
		};
	}, [showLoginDialog, pendingRequests.length, clearQueue]);

	return (
		<AuthContext.Provider
			value={{
				showLoginDialog,
				requireLogin,
				dismissLogin,
				handleLogin,
				pendingRequests,
				queueRequest,
				clearQueue,
				retryQueuedRequests,
				isRetrying,
				retryStatus,
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
