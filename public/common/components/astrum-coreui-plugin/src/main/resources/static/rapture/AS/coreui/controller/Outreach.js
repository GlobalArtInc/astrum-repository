
/*global Ext, AS*/

/**
 * Outreach controller.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.controller.Outreach', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.util.Url',
    'AS.State',
    'AS.Permissions'
  ],

  refs: [
    { ref: 'welcomePage', selector: 'as-dashboard-welcome' }
  ],

  /**
   * @override
   */
  init: function () {
    var me = this;

    me.listen({
      controller: {
        '#Refresh': {
          refresh: me.refreshOutreachContent
        },
        '#State': {
          userchanged: me.refreshOutreachContent
        },
        '#Permissions': {
          changed: me.refreshOutreachContent
        }
      },
      component: {
        'as-dashboard-welcome': {
          afterrender: me.refreshOutreachContent
        }
      }
    });
  },

  showOutreachWithProxyDownloads: function () {
    var me = this,
        proxyDownloadNumbers = "";
    AS.direct.outreach_Outreach.getProxyDownloadNumbers(function(response) {
      if (Ext.isObject(response) && response.success && response.data != null) {
        proxyDownloadNumbers = response.data;
      }
      me.showOutreach(proxyDownloadNumbers);
    });
  },

  showOutreach: function (proxyDownloadNumbers) {
    var me = this,
        welcomePage = me.getWelcomePage();

    AS.direct.outreach_Outreach.readStatus(function (response) {
      if (Ext.isObject(response) && response.success && response.data != null && welcomePage.rendered) {
        this.user = AS.State.getUser();
        var daysToExpiry = AS.State.getValue("license").daysToExpiry,
            databaseType = AS.State.getValue('datastore.isPostgresql') ? 'postgres' : 'h2',
            usertype,
            url,
            height;

        usertype = 'anonymous';
        height = '100%';

        if (this.user) {
          usertype = this.user.administrator ? 'admin' : 'normal';
        }

        url = AS.util.Url.urlOf('service/outreach/?version=' + AS.State.getVersion() +
            '&versionMm=' + AS.State.getVersionMajorMinor() +
            '&edition=' + AS.State.getEdition() +
            '&usertype=' + usertype +
            '&daysToExpiry=' + daysToExpiry +
            '&databaseType=' + databaseType +
            proxyDownloadNumbers
        );

        // add the outreach iframe to the welcome view
        welcomePage.add({
          xtype: 'uxiframe',
          itemId: 'outreach',
          anchor: '100%',
          width: '100%',
          height: height,
          flex: 1,
          border: false,
          frame: false,
          hidden: true,
          src: url,
          // override renderTpl to add title attribute for accessibility purpose
          renderTpl: [
            '<iframe src="{src}" id="{id}-iframeEl" data-ref="iframeEl" name="{frameName}" title="Astrum Repository Manager Outreach" width="100%" height="100%" frameborder="0"></iframe>'
          ],
          listeners: {
            load: function () {
              var iframe = this;
              // if the outreach content has loaded properly, show it
              if (iframe.getWin().iframeLoaded) {
                iframe.show();
              }
              else {
                // else complain and leave it hidden
                //<if debug>
                me.logDebug('Outreach iframe did not load: ' + url);
                //</if>
              }
            }
          }
        });
      }
    });
  },

  /**
   * @private
   * Add/Remove outreach content to/from welcome page, if outreach content is available.
   */
  refreshOutreachContent: function () {
    var me = this,
        welcomePage = me.getWelcomePage();

    if (welcomePage) {
      welcomePage.removeAll();
      me.showOutreachWithProxyDownloads();
    }
  }

});
