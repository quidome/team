Feature: Team configuration
  The coordinator configures the U16-1 team before recording memberships or program data.

  Scenario: Configuring U16-1
    Given no team named U16-1 is stored
    When the coordinator configures a team named U16-1
    Then the team is stored with the name U16-1
