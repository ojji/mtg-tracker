using System;
using System.IO;
using System.Threading.Tasks;
using UnityEngine;
using Wizards.Mtga.FrontDoorModels;
using Newtonsoft.Json;
using System.Threading;

namespace mtga_datacollector
{
  public class LogEntry
  {
    public string Timestamp;
    public object Attachment;
  }

  public class MtgaDataCollector : MonoBehaviour
  {
    private UnityCrossThreadLogger _logger = new UnityCrossThreadLogger("MTGADataCollector");
    private bool _subscribedToAccountInfo = false;
    private bool _subscribedToInventory = false;
    private bool _databaseLoaded = false;

    public void Start()
    {
      _logger.Info($"[initialization]Initialization started at {System.DateTime.Now:O}");
      Task initialize = new Task(Initialize);
      initialize.Start();
    }

    private void Initialize()
    {
      if (!_databaseLoaded && WrapperController.Instance != null && WrapperController.Instance.CardDatabase != null)
      {
        LoadDatabase();
      }
      if (!_subscribedToAccountInfo && WrapperController.Instance != null && WrapperController.Instance.AccountClient != null && WrapperController.Instance.AccountClient.AccountInformation != null && WrapperController.Instance.AccountClient.AccountInformation.AccountID != null)
      {
        SubscribeToAccountInfo();
      }
      if (!_subscribedToInventory && WrapperController.Instance != null && WrapperController.Instance.InventoryManager != null && WrapperController.Instance.InventoryManager.Cards != null && WrapperController.Instance.InventoryManager.Cards.Count > 0)
      {
        CollectInventoryAndSubscribeToChanges();
      }
      if (!_databaseLoaded || !_subscribedToInventory || !_subscribedToAccountInfo)
      {
        _logger.Info($"[initialization]Waiting for everyone to load {System.DateTime.Now:O}");
        System.Threading.Thread.Sleep(5000);
        Initialize();
      }

      _logger.Info($"[initialization]Initialization is done at {System.DateTime.Now:O}. Ready to go!");
    }

    private void LoadDatabase()
    {
      _logger.Info($"[card-db]{WrapperController.Instance.CardDatabase}");
      try
      {
        _logger.Info($"[card-db]Trying to write MID file");

        File.WriteAllText("f:\\temp\\mtga\\MID.txt", JsonConvert.SerializeObject(WrapperController.Instance.CardDatabase.GetPrintingsByExpansion("MID"), new JsonSerializerSettings
        {
          Formatting = Formatting.Indented,
          ReferenceLoopHandling = ReferenceLoopHandling.Serialize,
          PreserveReferencesHandling = PreserveReferencesHandling.Objects,
        }));

        _logger.Info($"[card-db]MID file written");

      }
      catch (Exception e)
      {
        _logger.Info($"[card-db]Something wrong: {e}");
      }

      _databaseLoaded = true;
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
        LogEntry cards = new LogEntry
        {
          Attachment = WrapperController.Instance.InventoryManager.Cards,
          Timestamp = String.Format($"{DateTime.Now:O}"),
        };

        LogEntry inventory = new LogEntry
        {
          Attachment = WrapperController.Instance.InventoryManager.Inventory,
          Timestamp = String.Format($"{DateTime.Now:O}"),
        };

        _logger.Info($"[collection]{JsonConvert.SerializeObject(cards)}");
        _logger.Info($"[inventory]{JsonConvert.SerializeObject(inventory)}");

      }

      Thread.Sleep(60000);
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
  }
}
