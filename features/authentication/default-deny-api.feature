Feature: Default-deny API access
  The coordinator tool protects application and data endpoints by default.

  Scenario: Anonymous client requests an unlisted API route
    Given no coordinator session exists
    When the client requests GET /api/players
    Then the response is 401 Unauthorized
    And the request does not reach the route handler

  Scenario: Authenticated coordinator requests an API route
    Given a valid coordinator session exists
    When the coordinator requests GET /api/players
    Then the request reaches the route handler

  Scenario: An operational health endpoint is requested
    Given no coordinator session exists
    When the client requests GET /api/health/liveness
    Then the request reaches the route handler
