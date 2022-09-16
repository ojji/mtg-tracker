using System;
using System.Threading.Tasks;
using UnityEngine;
using Wizards.Mtga.FrontDoorModels;
using Newtonsoft.Json;
using System.Threading;
using System.Linq;

namespace mtga_datacollector
{
  public class LogEntry
  {
    public string Timestamp;
    public object Attachment;
  }

  public class CardInventoryEntry
  {
    public uint grpId;
    public int count;
    public bool isRebalanced;
    public uint rebalancedCardLink;
  }
  public class MtgaDataCollector : MonoBehaviour
  {
    private UnityCrossThreadLogger _logger = new UnityCrossThreadLogger("MTGADataCollector");
    private bool _subscribedToAccountInfo = false;
    private bool _subscribedToInventory = false;

    public void Start()
    {
      _logger.Info($"[initialization]Initialization started at {System.DateTime.Now:O}");
      Task initialize = new Task(Initialize);
      initialize.Start();
    }

    private void Initialize()
    {
      if (!_subscribedToAccountInfo && WrapperController.Instance != null && WrapperController.Instance.AccountClient != null && WrapperController.Instance.AccountClient.AccountInformation != null && WrapperController.Instance.AccountClient.AccountInformation.AccountID != null)
      {
        SubscribeToAccountInfo();
      }
      if (!_subscribedToInventory && WrapperController.Instance != null && WrapperController.Instance.InventoryManager != null && WrapperController.Instance.InventoryManager.Cards != null && WrapperController.Instance.InventoryManager.Cards.Count > 0)
      {
        CollectInventoryAndSubscribeToChanges();
      }
      if (!_subscribedToInventory || !_subscribedToAccountInfo)
      {
        _logger.Info($"[initialization]Waiting for everyone to load {System.DateTime.Now:O}");
        System.Threading.Thread.Sleep(5000);
        Initialize();
      }

      _logger.Info($"[initialization]Initialization is done at {System.DateTime.Now:O}. Ready to go!");
    }

    private void SubscribeToAccountInfo()
    {
      WrapperController.Instance.AccountClient.LoginStateChanged += AccountClientLoginStateChanged;
      _logger.Info($"[account-info]{new { UserId = WrapperController.Instance.AccountClient.AccountInformation.AccountID, ScreenName = WrapperController.Instance.AccountClient.AccountInformation.DisplayName }}");
      _subscribedToAccountInfo = true;
    }

    private void AccountClientLoginStateChanged(LoginState obj)
    {
      _logger.Info($"[loginstate]{obj}");
      _subscribedToAccountInfo = false;
      _subscribedToInventory = false;
      Task.Run(Initialize);
    }

    private void CollectInventoryAndSubscribeToChanges()
    {
      WrapperController.Instance.InventoryManager.UnsubscribeFromAll(this.UpdateInventory);
      WrapperController.Instance.InventoryManager.SubscribeToAll(this.UpdateInventory);
      _subscribedToInventory = true;

      Task.Run(PeriodicUpdater);
    }

    private void PeriodicUpdater()
    {
      if (WrapperController.Instance != null && WrapperController.Instance.InventoryManager != null && WrapperController.Instance.InventoryManager.Cards != null && WrapperController.Instance.InventoryManager.Inventory != null)
      {
        try
        {
          var collection = WrapperController.Instance.InventoryManager.Cards.Select(pair =>
          {
            var cardPrinting = WrapperController.Instance.CardDatabase.GetPrintingByGrpId(pair.Key);
            return new CardInventoryEntry
            {
              grpId = pair.Key,
              count = pair.Value,
              isRebalanced = cardPrinting.IsRebalanced,
              rebalancedCardLink = cardPrinting.RebalancedCardLink
            };
          }).ToArray();

          var collectionEntry = new LogEntry
          {
            Attachment = collection,
            Timestamp = String.Format($"{DateTime.Now:O}"),
          };

          _logger.Info($"[collection]{JsonConvert.SerializeObject(collectionEntry)}");

          LogEntry inventory = new LogEntry
          {
            Attachment = WrapperController.Instance.InventoryManager.Inventory,
            Timestamp = String.Format($"{DateTime.Now:O}"),
          };

          _logger.Info($"[inventory]{JsonConvert.SerializeObject(inventory)}");
        }
        catch (Exception e)
        {
          _logger.Info($"[collection]{JsonConvert.SerializeObject(e)}");
        }
      }

      Thread.Sleep(TimeSpan.FromMinutes(30));
      PeriodicUpdater();
    }

    private void UpdateInventory(ClientInventoryUpdateReportItem payload)
    {
      LogEntry inventoryUpdate = new LogEntry
      {
        Timestamp = String.Format($"{DateTime.Now:O}"),
        Attachment = payload
      };

      _logger.Info($"[inventory-update]{JsonConvert.SerializeObject(inventoryUpdate)}");
    }

    public void OnDestroy()
    {
      _logger.Info($"[initialization]Shutting down at {System.DateTime.Now:O}. Bye!");
    }

    public void OnDisable()
    {
      _logger.Info($"[initialization]Disabled at {System.DateTime.Now:O}. Oops!");
    }

    public void OnApplicationQuit()
    {
      _logger.Info($"[initialization]App quit at {System.DateTime.Now:O}. Bye bye!");
    }
  }
}
