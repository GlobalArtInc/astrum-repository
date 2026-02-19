
package org.globalart.astrum.repository.rest.api;

import java.util.Base64;
import jakarta.ws.rs.NotFoundException;
import jakarta.ws.rs.WebApplicationException;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.http.HttpStatus.UNPROCESSABLE_ENTITY;

/**
 * An object to hold the id and repository id of an asset in the format repository-id:asset-id, encoded.
 */
public class RepositoryItemIDXO
{
  public static final Logger log = LoggerFactory.getLogger(RepositoryItemIDXO.class);

  private String repositoryId;

  private String id;

  public RepositoryItemIDXO(String repositoryId, String id) {
    this.repositoryId = checkNotNull(repositoryId);
    this.id = checkNotNull(id);
  }

  public String getRepositoryId() {
    return repositoryId;
  }

  public void setRepositoryId(String repositoryId) {
    this.repositoryId = repositoryId;
  }

  public String getId() {
    return id;
  }

  public void setId(String id) {
    this.id = id;
  }

  public static RepositoryItemIDXO fromString(String encoded) {
    try {
      String decoded = new String(Base64.getUrlDecoder().decode(encoded));
      String[] parts = decoded.split(":");
      if (parts.length != 2) {
        throw new WebApplicationException("Unable to parse RepositoryItemIDXO " + encoded, UNPROCESSABLE_ENTITY);
      }
      return new RepositoryItemIDXO(parts[0], parts[1]);
    }
    catch (IllegalArgumentException e) {
      log.debug("Unable to parse id: {}, returning 404.", encoded, e);
      throw new NotFoundException("Unable to locate asset with id " + encoded);
    }
  }

  public String getValue() {
    return new String(Base64.getUrlEncoder().withoutPadding().encode((repositoryId + ":" + id).getBytes()));
  }
}
