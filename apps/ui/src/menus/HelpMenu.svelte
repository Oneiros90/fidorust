<script lang="ts">
	import { getAppSession } from '../app/appContext';
	import { LATEST_RELEASE_URL, NEW_ISSUE_URL, openGitHubUrl } from '../lib/updates';
	import MenuItem from './MenuItem.svelte';
	import MenuSeparator from './MenuSeparator.svelte';

	const app = getAppSession();
</script>

<MenuItem label={app.t.about} onclick={app.openAbout} />
<MenuItem label={app.t.technologies} onclick={app.openTechnologies} />
<MenuSeparator />
<MenuItem label={app.t.proEnter} onclick={app.pro.tryEnter} />
<MenuSeparator />
<MenuItem label={app.t.reportIssue} onclick={() => void openGitHubUrl(NEW_ISSUE_URL)} />
<MenuItem label={app.t.downloadDesktop} onclick={() => void openGitHubUrl(LATEST_RELEASE_URL)} />
