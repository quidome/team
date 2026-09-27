Feature: Coordinator teams
  Each person who logs in is a coordinator, identified by the Pocket ID subject.
  A coordinator is responsible for one or more teams and has one default team.
  A team can have more than one coordinator. Roles and per-team access checks are not part of this feature.

  Scenario: Recording a coordinator at first login
    Given no coordinator is stored for the Pocket ID subject of Sam
    When Sam completes a login
    Then a coordinator is stored for that subject
    And the coordinator is responsible for no teams yet

  Scenario: Recognising a returning coordinator
    Given a coordinator is stored for the Pocket ID subject of Sam
    When Sam completes a login again
    Then no second coordinator is stored for that subject

  Scenario: Taking responsibility for more than one team
    Given Sam is a coordinator
    When Sam takes responsibility for U16-1 and U16-2
    Then Sam's teams are U16-1 and U16-2

  Scenario: Sharing a team between coordinators
    Given Sam is responsible for U16-1
    When Robin takes responsibility for U16-1
    Then the coordinators of U16-1 are Sam and Robin

  Scenario: Setting the default team
    Given Sam is responsible for U16-1 and U16-2
    When Sam sets U16-1 as the default team
    Then U16-1 is Sam's default team

  Scenario: Rejecting a default team outside the coordinator's teams
    Given Sam is responsible for U16-1 only
    When Sam sets U18-1 as the default team
    Then the change is rejected because Sam is not responsible for U18-1

  Scenario: Opening the default team after login
    Given Sam's default team is U16-1
    When Sam completes a login
    Then the U16-1 program is shown

  Scenario: Giving up responsibility for the default team
    Given Sam is responsible for U16-1 and U16-2 with U16-1 as the default team
    When Sam gives up responsibility for U16-1
    Then Sam has no default team until Sam chooses one

  Scenario: Recording who made a change
    Given Sam is logged in as a coordinator
    When Sam assigns Avery to a jury place
    Then the duty history shows that Sam made the assignment

  Scenario: Keeping a coordinator profile
    Given Sam is a coordinator
    When Sam sets the display name Sam Jansen and the email sam@example.test
    Then Sam's profile shows the display name Sam Jansen and the email sam@example.test
