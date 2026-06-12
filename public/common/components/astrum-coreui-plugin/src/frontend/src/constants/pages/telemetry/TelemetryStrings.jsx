

const HELPER_LINK = {
  TEXT: 'base telemetry',
  HREF: 'https://links.globalart.dev/products/astrum/astrum-telemetry',
}

const SHARED = {
  GENERAL_RECOMMENDATION: 'Check your network configuration and verify that the server can reach the telemetry service. If the problem persists, contact GlobalArt Support.',
  RETRY_RECOMMENDATION: 'To retry the upload now, run the Upload Retry task.',
  RETRY_LINK: 'Go to Tasks.',
  HELPER_LINK,
}

export default {
  TELEMETRY: {
    INTRODUCED_WARNING_BANNER: {
      TITLE: 'Baseline Telemetry Upload Issue',
      MESSAGE: 'This instance of Astrum Repository has been unable to upload required base telemetry data for the past {failedReportDays} days. This is a warning only. Your instance will continue to operate normally.',
      GENERAL_RECOMMENDATION: 'Check that your server can reach the GlobalArt telemetry service, then run the Upload Retry task.',
      RETRY_RECOMMENDATION: 'If your environment cannot send telemetry or the issue persists, contact GlobalArt Support.',
      RETRY_LINK: 'Go to Tasks.',
      HELPER_LINK,
    },
    WARNING_BANNER: {
      TITLE: 'Baseline Telemetry Upload Issue',
      MESSAGE: 'Astrum Repository has been unable to upload base telemetry data. If telemetry uploads cannot be restored, Astrum Repository will enter **read-only mode in {remainingGracePeriodDays} day(s)**.',
      ...SHARED,
    },
    READONLY_BANNER: {
      TITLE: 'Read-Only Mode Enabled',
      MESSAGE: 'Astrum Repository is now in read-only mode because base telemetry data could not be uploaded within the grace period.',
      ...SHARED,
    },
  },
}
