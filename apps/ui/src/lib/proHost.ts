export type ProHost = {
	t: {
		proActive: string;
		proActiveBody: string;
		proDeactivate: string;
		close: string;
	};
	deactivate: () => void;
	exit: () => void;
	extCommand: (name: string, payload: string) => string;
	subscribeRefresh: (cb: () => void) => () => void;
};

export type ProPack = {
	mount: (target: HTMLElement, host: ProHost) => () => void;
	setCircuitChrome: (on: boolean) => void;
};
