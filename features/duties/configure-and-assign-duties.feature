Feature: Coordinate game duties and transport
  The coordinator configures duty slots, records volunteers, and confirms assignments.

  Scenario: Configuring away-game driving and officiating slots
    Given a scheduled U16-1 away game at 14:30
    When the coordinator configures one driving slot, one referee slot, and one jury slot
    Then the game has those duty slots
    And the suggested departure time remains visible

  Scenario: Retaining volunteers while reassigning a duty
    Given two players volunteered for one referee slot
    When the coordinator assigns the slot to Avery and later reassigns it to Blake
    Then both volunteer signups remain in the duty history
    And only Blake receives completion credit after the game
