type Theme = 'light' | 'dark' | 'system';

let mediaQuery: MediaQueryList | null = null;
let mediaHandler: ((e: MediaQueryListEvent) => void) | null = null;

function resolveTheme(theme: Theme): 'light' | 'dark' {
	if (theme === 'system') {
		if (!mediaQuery) {
			mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
		}
		return mediaQuery.matches ? 'dark' : 'light';
	}
	return theme;
}

function apply(resolved: 'light' | 'dark') {
	document.documentElement.setAttribute('data-theme', resolved);
}

export function applyTheme(theme: Theme) {
	// Clean up previous listener
	if (mediaHandler && mediaQuery) {
		mediaQuery.removeEventListener('change', mediaHandler);
		mediaHandler = null;
	}

	if (theme === 'system') {
		mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
		apply(mediaQuery.matches ? 'dark' : 'light');

		mediaHandler = (e: MediaQueryListEvent) => {
			apply(e.matches ? 'dark' : 'light');
		};
		mediaQuery.addEventListener('change', mediaHandler);
	} else {
		apply(resolveTheme(theme));
	}
}
