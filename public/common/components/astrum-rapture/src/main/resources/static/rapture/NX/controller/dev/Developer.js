
/*global Ext, NX, console*/

/**
 * Developer controller.
 *
 * @since 3.0
 */
Ext.define('NX.controller.dev.Developer', {
  extend: 'NX.app.Controller',
  requires: [
    'Ext.state.Manager',
    'NX.State',
    'NX.Messages'
  ],

  _OVER_LIMITS: 'Over limits',
  _NEAR_LIMITS: '75% usage',
  _UNDER_LIMITS: 'Under limits',

  // Metric data constants
  _DEFAULT: 0,
  _COMPONENTS_NEAR_LIMITS: 75000,
  _REQUESTS_NEAR_LIMITS: 150000,
  _COMPONENTS_OVER_LIMITS: 40000,
  _REQUESTS_OVER_LIMITS: 100000,
  _REQUESTS_HARD_THRESHOLD: 100000,
  _REQUESTS_SOFT_THRESHOLD: 20000,
  _COMPONENTS_HARD_THRESHOLD: 40000,
  _COMPONENTS_SOFT_THRESHOLD: 100000,
  _UNIQUE_USER_SOFT_THRESHOLD: 100,
  _GRACE_PERIOD_DAYS_IN: 7,
  _GRACE_PERIOD_DAYS_POST: -7,

  /**
   * Attempts to call a object's method that doesn't exist to produce a low-level javascript error.
   *
   * @private
   */
  testError: function () {
    console.log_no_such_method();
  },

  /**
   * Raises an Ext.Error so we can see how that behaves.
   *
   * @private
   */
  testExtError: function () {
    Ext.Error.raise('simulated error');
  },

  /**
   * Adds messages for each of the major types to view styling, etc.
   *
   * @private
   */
  testMessages: function () {
    NX.Messages.success('Success');
    NX.Messages.info( 'Test of a long info message. Lorem ipsum dolor sit amet, consectetur adipiscing elit, ' +
        'sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.');
    NX.Messages.warning('A warning test');
    NX.Messages.error('Test of an error message');
  },

  /**
   * Toggle the unsupported browser application state.
   *
   * @private
   */
  toggleUnsupportedBrowser: function() {
    NX.State.setBrowserSupported(!NX.State.isBrowserSupported());
  },

  /**
   * Modify state so that the database quorum warning is shown in the UI.
   *
   * @private
   */
  showQuorumWarning: function () {
    NX.State.setValue('quorum', { 'quorumPresent': false});
  },

  /**
   * Clear local browser state.
   *
   * @private
   */
  clearLocalState: function() {
    var provider = Ext.state.Manager.getProvider();
    // HACK: provider.state is private
    Ext.Object.each(provider.state, function (key, value) {
      provider.clear(key);
    });
  },

  /**
   * Can be used to disconnect the state updates at runtime for debugging
   */
  toggleStateUpdates: function() {
    var state = NX.State.controller();
    if (state.statusProvider.isConnected()) {
      state.statusProvider.disconnect();
      NX.Messages.warning('State updates disconnected');
    }
    else {
      state.statusProvider.connect();
      NX.Messages.info('State updates re-connected');
    }
  },

  //-- Simulate various Community Edition States, disconnect state updates first --//

  showUnderLimitsPreGracePeriod: function() {
    NX.State.setValues({
      contentUsageEvaluationResult: this.mockMetricData(),
      'astrum.community.gracePeriodEnds': '',
      'astrum.community.throttlingStatus': this._UNDER_LIMITS
    });
  },

  showNearLimitsPreGracePeriod: function() {
    NX.State.setValues({
      contentUsageEvaluationResult: this.mockMetricData(this._COMPONENTS_NEAR_LIMITS, this._REQUESTS_NEAR_LIMITS),
      'astrum.community.gracePeriodEnds': '',
      'astrum.community.throttlingStatus': this._NEAR_LIMITS
    });
  },

  showUnderLimitsInGracePeriod: function() {
    NX.State.setValues({
      contentUsageEvaluationResult: this.mockMetricData(),
      'astrum.community.gracePeriodEnds': this.relativeGracePeriodDate(this._GRACE_PERIOD_DAYS_IN),
      'astrum.community.throttlingStatus': this._UNDER_LIMITS
    });
  },

  showNearLimitsInGracePeriod: function() {
    NX.State.setValues({
      contentUsageEvaluationResult: this.mockMetricData(this._COMPONENTS_NEAR_LIMITS, this._REQUESTS_NEAR_LIMITS),
      'astrum.community.gracePeriodEnds': this.relativeGracePeriodDate(this._GRACE_PERIOD_DAYS_IN),
      'astrum.community.throttlingStatus': this._NEAR_LIMITS
    });
  },

  showOverLimitsInGracePeriod: function() {
    NX.State.setValues({
      contentUsageEvaluationResult: this.mockMetricData(this._COMPONENTS_OVER_LIMITS, this._REQUESTS_OVER_LIMITS),
      'astrum.community.gracePeriodEnds': this.relativeGracePeriodDate(this._GRACE_PERIOD_DAYS_IN),
      'astrum.community.throttlingStatus': this._OVER_LIMITS
    });
  },

  showUnderLimitsPostGracePeriod: function() {
    NX.State.setValues({
      contentUsageEvaluationResult: this.mockMetricData(),
      'astrum.community.gracePeriodEnds': this.relativeGracePeriodDate(this._GRACE_PERIOD_DAYS_POST),
      'astrum.community.throttlingStatus': this._UNDER_LIMITS
    });
  },

  showNearLimitsPostGracePeriod: function() {
    NX.State.setValues({
      contentUsageEvaluationResult: this.mockMetricData(this._COMPONENTS_NEAR_LIMITS, this._REQUESTS_NEAR_LIMITS),
      'astrum.community.gracePeriodEnds': this.relativeGracePeriodDate(this._GRACE_PERIOD_DAYS_POST),
      'astrum.community.throttlingStatus': this._NEAR_LIMITS
    });
  },

  showOverLimitsPostGracePeriod: function() {
    NX.State.setValues({
      contentUsageEvaluationResult: this.mockMetricData(this._COMPONENTS_OVER_LIMITS, this._REQUESTS_OVER_LIMITS),
      'astrum.community.gracePeriodEnds': this.relativeGracePeriodDate(this._GRACE_PERIOD_DAYS_POST),
      'astrum.community.throttlingStatus': this._OVER_LIMITS
    });
  },

  mockMetricData: function(components, requests) {
    // below limits
    return [
      {
        "metricName": "peak_requests_per_day",
        "metricValue": requests || this._DEFAULT,
        "thresholds": [
          {"thresholdName": "HARD_THRESHOLD", "thresholdValue": this._REQUESTS_HARD_THRESHOLD},
          {"thresholdName": "SOFT_THRESHOLD", "thresholdValue": this._REQUESTS_SOFT_THRESHOLD}
        ],
        "utilization": "FREE_TIER",
        "aggregates": [{"name": "content_request_count", "value": requests || this._DEFAULT, "period": "peak_recorded_count_30d"}]
      }, {
        "metricName": "component_total_count",
        "metricValue": components || this._DEFAULT,
        "thresholds": [
          {"thresholdName": "HARD_THRESHOLD", "thresholdValue": this._COMPONENTS_HARD_THRESHOLD},
          {"thresholdName": "SOFT_THRESHOLD", "thresholdValue": this._COMPONENTS_SOFT_THRESHOLD}
        ],
        "utilization": "FREE_TIER",
        "aggregates": [{"name": "component_total_count", "value": components || this._DEFAULT, "period": "peak_recorded_count_30d"}]
      }, {
        "metricName": "successful_last_24h",
        "metricValue": this._DEFAULT,
        "thresholds": [{"thresholdName": "SOFT_THRESHOLD", "thresholdValue": this._UNIQUE_USER_SOFT_THRESHOLD}],
        "utilization": "FREE_TIER",
        "aggregates": [{"name": "unique_user_count", "value": this._DEFAULT, "period": "peak_recorded_count_30d"}]
      }
    ];
  },

  relativeGracePeriodDate: function (dayOffset) {
    var nextWeek = new Date();
    nextWeek.setDate(new Date().getDate() + dayOffset);
    return nextWeek.toISOString();
  }
});
