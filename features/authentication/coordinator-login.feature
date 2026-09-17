Feature: Coordinator login
  The sole MVP coordinator signs in through Pocket ID, while this application owns the session.

  Scenario: Starting a login
    Given Pocket ID client configuration is available
    When the coordinator requests GET /auth/login
    Then the application stores a short-lived authorization transaction
    And redirects the coordinator with PKCE, state, and nonce protections

  Scenario: Completing a login
    Given a valid unexpired authorization transaction exists
    And Pocket ID returns a validated subject to GET /auth/callback
    Then the application creates a secure HTTP-only coordinator session
    And redirects the coordinator to the home page

  Scenario: Logging out
    Given a coordinator session exists
    When the coordinator sends POST /auth/logout
    Then the application removes the coordinator session
    And redirects the coordinator to the home page
