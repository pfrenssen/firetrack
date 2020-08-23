Feature: Expenses
  In order to keep track of my expenses
  As a user
  I want to enter, manage and view my expenses

  Scenario: Add an expense
    Given I am logged in as "pallas.park@email.gr"
    And I click "Expenses" in the "sidebar navigation"
    Then I should see the heading "Expenses"
    And I should be on "/expenses"
    When I click "Add expense"
    Then I should see the heading "Add expense"
    And I should be on "/expenses/add"
