
/*global Ext, NX*/

/**
 * Registers all feature groups for coreui.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.controller.FeatureGroups', {
  extend: 'NX.app.Controller',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  init: function () {
    this.getApplication().getFeaturesController().registerFeature([
      {
        mode: 'admin',
        path: '/Repository',
        text: NX.I18n.get('FeatureGroups_Repository_Text'),
        description: NX.I18n.get('FeatureGroups_Repository_Description'),
        group: true,
        weight: 50,
        iconConfig: {
          file: 'database.png',
          variants: ['x16', 'x32']
        }
      },
      {
        mode: 'admin',
        path: '/Security',
        text: NX.I18n.get('FeatureGroups_Security_Title'),
        description: NX.I18n.get('FeatureGroups_Security_Description'),
        group: true,
        weight: 90,
        iconConfig: {
          file: 'security.png',
          variants: ['x16', 'x32']
        }
      },
      {
        mode: 'admin',
        path: '/Support',
        text: NX.I18n.get('FeatureGroups_Support_Text'),
        description: NX.I18n.get('FeatureGroups_Support_Description'),
        group: true,
        expanded: false,
        iconConfig: {
          file: 'support.png',
          variants: ['x16', 'x32']
        }
      },
      {
        mode: 'admin',
        path: '/System',
        text: NX.I18n.get('FeatureGroups_System_Text'),
        description: NX.I18n.get('FeatureGroups_System_Description'),
        group: true,
        expanded: false,
        weight: 1000,
        iconConfig: {
          file: 'cog.png',
          variants: ['x16', 'x32']
        }
      },
    ]);
  }
});
