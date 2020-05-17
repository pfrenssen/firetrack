Feature: Error pages
  In order to deliver a coherent browsing experience
  As a product owner
  I want to present error pages in HTML format

  Scenario: Anonymous and authenticated users can see 404 pages
    When I go to "/non-existing-path"
    Then the response status code should be 404
    And I should see the heading "Page not found"
    And I should see "404"
    And I should see the link "Sign up"
    And I should see the link "Log in"
    But I should not see the link "Log out"

    Given I am logged in as "eleonora.giffard@mail.co.uk"
    When I go to "/non-existing-path"
    Then the response status code should be 404
    And I should see the heading "Page not found"
    And I should see "404"
    And I should see the link "Log out"
    But I should not see the link "Sign up"
    And I should not see the link "Log in"

  Scenario: Anonymous and authenticated users can see 403 pages
    # Anonymous users get a 403 on the activation page.
    When I go to "/user/activate"
    Then the response status code should be 403
    And I should see the heading "Access denied"
    And I should see "403"
    And I should see the link "Sign up"
    And I should see the link "Log in"
    But I should not see the link "Log out"

    # Authenticated users get a 403 on the login page.
    Given I am logged in as "malka.gonzales@gandi.net"
    When I go to "/user/login"
    Then the response status code should be 403
    And I should see the heading "Access denied"
    And I should see "403"
    And I should see the link "Log out"
    But I should not see the link "Sign up"
    And I should not see the link "Log in"
