
package org.globalart.astrum.repository.rest.internal.resources;

import java.net.URI;
import java.util.List;

import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.Response.ResponseBuilder;
import jakarta.ws.rs.core.Response.Status;
import jakarta.ws.rs.core.UriBuilder;

import org.globalart.astrum.repository.rest.api.AssetXO;
import org.globalart.astrum.rest.WebApplicationMessageException;

import static jakarta.ws.rs.core.Response.Status.BAD_REQUEST;
import static jakarta.ws.rs.core.Response.Status.NOT_FOUND;

/**
 * A helper class to produce an appropriate HTTP Response for asset download requests.
 *
 * @since 3.7
 */
public class AssetDownloadResponseProcessor
{
  public static final String NO_SEARCH_RESULTS_FOUND = "Asset search returned no results";

  public static final String SEARCH_RETURNED_MULTIPLE_ASSETS =
      "Search returned multiple assets, please refine search criteria to find a single asset or use the sort query parameter to retrieve the first result.";

  private final List<AssetXO> assetXOs;

  private final boolean sorted;

  AssetDownloadResponseProcessor(final List<AssetXO> assetXOs, boolean sorted) {
    this.assetXOs = assetXOs;
    this.sorted = sorted;
  }

  /**
   * Produces the appropriate http response based upon the assets list. Initial
   * implementation is returning a BAD_REQUEST when collection has more than one asset
   * as there is no means for determining the appropriate asset in the absence of a
   * selection strategy (future work).
   * 
   * @return response the appropriate {@link Response} based on the input asset list
   */
  Response process() {
    if (assetXOs.isEmpty()) {
      throw new WebApplicationMessageException(NOT_FOUND, NO_SEARCH_RESULTS_FOUND);
    }

    // if there more than 1 result, and no sort, we throw an error
    if (assetXOs.size() > 1 && !sorted) {
      throw new WebApplicationMessageException(BAD_REQUEST, SEARCH_RETURNED_MULTIPLE_ASSETS);
    }

    // if only 1 result, or sorting enabled, return the 1st one
    return getResponse(assetXOs.get(0));
  }

  /**
   * Builds the response with the provided assetXO object
   * 
   * @param assetXO the {@link AssetXO} to be downloaded
   * @return response the redirect {@link Response} containing the download url
   */
  private Response getResponse(final AssetXO assetXO) {
    String redirectUrl = assetXO.getDownloadUrl();
    URI uri = UriBuilder.fromPath(redirectUrl).build();
    ResponseBuilder builder = Response.status(Status.FOUND).location(uri);
    return builder.build();
  }
}
