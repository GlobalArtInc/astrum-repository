
/*global Ext, AS*/

/**
 * Help controller.
 *
 * @since 3.0
 */
Ext.define('AS.controller.Help', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.Icons',
    'AS.I18n',
    'AS.Windows'
  ],

  views: [
    'header.Help',
    'AboutWindow'
  ],

  statics: {
    /**
     * The base-url for help links.
     *
     * @private
     * @property {String}
     * @readonly
     */
    baseUrl: 'https://links.globalart.dev/products/astrum',

    getDocsUrl: function() {
      return AS.controller.Help.baseUrl + '/docs/' + AS.State.getVersionMajorMinor();
    }
  },

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.getApplication().getIconController().addIcons({
      'help-support': {
        file: 'support.png',
        variants: ['x16', 'x32']
      },
      'help-issues': {
        file: 'bug.png',
        variants: ['x16', 'x32']
      },
      'help-manual': {
        file: 'book_picture.png',
        variants: ['x16', 'x32']
      },
      'help-community': {
        file: 'users_4.png',
        variants: ['x16', 'x32']
      },
      'help-kb': {
        file: 'brain_trainer.png',
        variants: ['x16', 'x32']
      },
      'help-guides': {
        file: 'globalart.png',
        variants: ['x16', 'x32']
      }
    });

    me.listen({
      component: {
        'as-header-help menuitem[action=about]': {
          click: me.onAbout
        },
        'as-header-help menuitem[action=docs]': {
          click: me.onDocs
        },
        'as-header-help menuitem[action=support]': {
          click: me.onSupport
        },
        'as-header-help menuitem[action=issues]': {
          click: me.onIssues
        },
        'as-header-help menuitem[action=community]': {
          click: me.onCommunity
        },
        'as-header-help menuitem[action=kb]': {
          click: me.onKnowledgeBase
        },
        'as-header-help menuitem[action=guides]': {
          click: me.onGuides
        }
      }
    });
  },

  /**
   * @private
   * @param {String} section
   */
  openUrl: function(section) {
    AS.Windows.open(AS.controller.Help.baseUrl + '/' + section);
  },

  /**
   * @private
   */
  onAbout: function() {
    Ext.widget('as-aboutwindow');
  },

  /**
   * @private
   */
  onDocs: function() {
    AS.Windows.open(AS.controller.Help.getDocsUrl());
  },

  /**
   * @private
   */
  onSupport: function() {
    this.openUrl('support');
  },

  /**
   * @private
   */
  onIssues: function() {
    this.openUrl('issues');
  },

  /**
   * @private
   */
  onCommunity: function() {
    this.openUrl('community');
  },

  /**
   * @private
   */
  onKnowledgeBase: function() {
    this.openUrl('kb');
  },

  /**
   * @private
   */
  onGuides: function() {
    AS.Windows.open("https://links.globalart.dev/products/astrum/guides")
  }
});
