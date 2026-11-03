export type ProHost = {
	t: {
		proActive: string;
		proActiveBody: string;
		proDeactivate: string;
		close: string;
		delete: string;
	};
	deactivate: () => void;
	exit: () => void;
	extCommand: (name: string, payload: string) => string;
	subscribeRefresh: (cb: () => void) => () => void;
	bindPadLinkContext: (fn: (clientX: number, clientY: number) => boolean) => () => void;
};

export type ProPack = {
	mount: (target: HTMLElement, host: ProHost) => () => void;
	mountStage?: (target: HTMLElement, host: ProHost) => () => void;
	setCircuitChrome: (on: boolean) => void;
};
