
/*global Ext, AS*/

/**
 * Registers all feature groups for coreui.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.controller.FeatureGroups', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  init: function () {
    this.getApplication().getFeaturesController().registerFeature([
      {
        mode: 'admin',
        path: '/Repository',
        text: AS.I18n.get('FeatureGroups_Repository_Text'),
        description: AS.I18n.get('FeatureGroups_Repository_Description'),
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
        text: AS.I18n.get('FeatureGroups_Security_Title'),
        description: AS.I18n.get('FeatureGroups_Security_Description'),
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
        text: AS.I18n.get('FeatureGroups_Support_Text'),
        description: AS.I18n.get('FeatureGroups_Support_Description'),
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
        text: AS.I18n.get('FeatureGroups_System_Text'),
        description: AS.I18n.get('FeatureGroups_System_Description'),
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
