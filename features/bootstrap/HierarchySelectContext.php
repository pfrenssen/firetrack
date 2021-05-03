<?php

declare(strict_types = 1);

use Behat\Mink\Element\NodeElement;
use Behat\Mink\Element\TraversableElement;
use Behat\MinkExtension\Context\RawMinkContext;
use Firetrack\Tests\Exception\ExpectationException;
use Firetrack\Tests\Traits\KeyboardInteractionTrait;

/**
 * Step definitions for interacting with the Hierarchy Select jQuery plugin.
 *
 * Ref. https://github.com/NeoFusion/hierarchy-select
 */
class HierarchySelectContext extends RawMinkContext
{

    use KeyboardInteractionTrait;

    /**
     * Expands the hierarchical select dropdown with the given label.
     *
     * This performs the selection by typing the option in the search box and then pressing "down" followed by "enter"
     * to select the topmost option in the dropdown.
     *
     * @param string $label
     *
     * @throws \Exception
     *   Thrown when the hierarchy select element is not found.
     *
     * @When I select the :option option in the :label hierarchical dropdown
     */
    public function select(string $label, string $option): void
    {
        $this->expand($label);

        $search_box_element = $this->getSearchBoxElement($label);
        $search_box_element->setValue($option);
        $this->pressKeyInElement('down', $search_box_element);
        $this->pressKeyInElement('enter', $search_box_element);
    }

    /**
     * Expands the hierarchical select dropdown with the given label.
     *
     * @param string $label
     *
     * @throws \Exception
     *   Thrown when the hierarchy select element is not found or does not
     *   expand.
     *
     * @When I expand the :label hierarchical dropdown
     */
    public function expand(string $label): void
    {
        if (!$this->isExpanded($label)) {
            $this->toggle($label);
            // Wait for the search box to become visible.
            for ($i = 0; $i < 200; $i++) {
              $search_box = $this->getSearchBoxElement($label);
              if ($search_box->isVisible()) {
                return;
              }
              usleep(50000);
            }
            throw new \Exception('Hierarchical select element did not expand.');
        }
    }

    /**
     * Collapses the hierarchical select dropdown with the given label.
     *
     * @param string $label
     *
     * @throws \Exception
     *   Thrown when the hierarchy select element is not found.
     *
     * @When I collapse the :label hierarchical dropdown
     */
    public function collapse(string $label): void
    {
        if ($this->isExpanded($label)) {
            $this->toggle($label);
        }
    }

    /**
     * Checks that the hierarchical dropdown with the given label is expanded.
     *
     * @param string $label
     *
     * @throws \Exception
     *   Thrown when the hierarchy select element is not found.
     *
     * @Then the :label hierarchical dropdown should be expanded
     */
    public function assertExpanded(string $label): void
    {
        if (!$this->isExpanded($label)) {
            throw new ExpectationException("The '$label' hierarchical select is not expanded, but it was expected to be.");
        }
    }

    /**
     * Checks that the hierarchical dropdown with the given label is not expanded.
     *
     * @param string $label
     *
     * @throws \Exception
     *   Thrown when the hierarchy select element is not found.
     *
     * @Then the :label hierarchical dropdown should not be expanded
     */
    public function assertNotExpanded(string $label): void
    {
        if ($this->isExpanded($label)) {
            throw new ExpectationException("The '$label' hierarchical select is expanded, but it was not expected to be.");
        }
    }

    /**
     * Retrieves a hierarchy select element by label.
     *
     * This returns the container element that is tagged with the "hierarchy-select" class.
     *
     * @param string $label
     * @param \Behat\Mink\Element\TraversableElement|null $region
     *   (optional) The region in which to search for the hierarchy select element. Defaults to the entire page.
     *
     * @return \Behat\Mink\Element\NodeElement
     *
     * @throws \Exception
     *   Thrown when the hierarchy select element is not found.
     */
    protected function getHierarchySelectElement(string $label, ?TraversableElement $region = null): NodeElement
    {
        if (empty($region)) {
            $region = $this->getSession()->getPage();
        }
        $xpath = "//div[@id=(//label[text()='$label']/@for) and contains(concat(' ', normalize-space(@class), ' '), ' hierarchy-select ')]";
        /** @var \Behat\Mink\Element\NodeElement $element */
        $element = $region->find('xpath', $xpath);

        if (empty($element)) {
            throw new Exception("Hierarchy select field '{$label}' not found.");
        }

        return $element;
    }

    /**
     * Returns the dropdown toggle button for the hierarchy select field with the given label.
     *
     * @param string $label
     * @param \Behat\Mink\Element\TraversableElement|null $region
     *   (optional) The region in which to search for the hierarchy select element. Defaults to the entire page.
     *
     * @return \Behat\Mink\Element\NodeElement
     *
     * @throws \Exception
     *   Thrown when the hierarchy select element or its toggle button is not found.
     */
    protected function getDropdownToggleElement(string $label, ?TraversableElement $region = null): NodeElement
    {
        $hierarchy_select_element = $this->getHierarchySelectElement($label, $region);
        $xpath = '//button[contains(concat(" ", normalize-space(@class), " "), " dropdown-toggle ")]';
        $element = $hierarchy_select_element->find('xpath', $xpath);

        if (!$element instanceof NodeElement) {
            throw new Exception("Dropdown toggle button for hierarchy select field '{$label}' not found.");
        }

        return $element;
    }

    /**
     * Returns the search box for the hierarchy select field with the given label.
     *
     * @param string $label
     * @param \Behat\Mink\Element\TraversableElement|null $region
     *   (optional) The region in which to search for the hierarchy select element. Defaults to the entire page.
     *
     * @return \Behat\Mink\Element\NodeElement
     *
     * @throws \Exception
     *   Thrown when the hierarchy select element or its search box is not found.
     */
    protected function getSearchBoxElement(string $label, ?TraversableElement $region = null): NodeElement
    {
        $hierarchy_select_element = $this->getHierarchySelectElement($label, $region);
        $element = $hierarchy_select_element->find('css', '.hs-searchbox input.form-control');

        if (!$element instanceof NodeElement) {
            throw new Exception("Search box for hierarchy select field '{$label}' not found.");
        }

        return $element;
    }

    /**
     * Checks if the hierarchy select element with the given label is expanded.
     *
     * @param string $label
     * @param \Behat\Mink\Element\TraversableElement|null $region
     *   (optional) The region in which to search for the hierarchy select element. Defaults to the entire page.
     *
     * @return bool
     *
     * @throws \Exception
     *   Thrown when the hierarchy select element is not found.
     */
    protected function isExpanded(string $label, ?TraversableElement $region = null): bool
    {
        $dropdown_toggle_element = $this->getDropdownToggleElement($label, $region);
        if (!$dropdown_toggle_element->hasAttribute('aria-expanded')) {
            throw new \Exception("Could not determine whether the hierarchy select field '$label' is expanded.");
        }
        return $dropdown_toggle_element->getAttribute('aria-expanded') === 'true';
    }

    /**
     * Toggles the visibility of the hierarchy select dropdown with the given label.
     *
     * @param string $label
     * @param \Behat\Mink\Element\TraversableElement|null $region
     *   (optional) The region in which to search for the hierarchy select element. Defaults to the entire page.
     *
     * @throws \Exception
     *   Thrown when the hierarchy select element is not found.
     */
    protected function toggle(string $label, ?TraversableElement $region = null): void
    {
        $this->getDropdownToggleElement($label, $region)->click();
    }

}
